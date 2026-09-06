use crate::utils::{resolve_context_path, resolve_safe_path};
use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFileNamesConfig {
    /// Directory to scan
    pub directory: String,
    /// Optional regex pattern to filter filenames (matches against filename only, not full path)
    #[serde(default)]
    pub pattern: Option<String>,
    /// Recurse into subdirectories
    #[serde(default)]
    pub include_subdirs: bool,
}

pub struct GetFileNames {
    config: GetFileNamesConfig,
    resolved_dir: Option<String>,
}

impl GetFileNames {
    pub fn new(config: GetFileNamesConfig) -> Self {
        Self {
            config,
            resolved_dir: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: GetFileNamesConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn schema() -> Arc<RowSchema> {
        Arc::new(RowSchema::new(vec![
            Field::new("filename", ValueType::String),
            Field::new("filepath", ValueType::String),
            Field::new("filesize", ValueType::Integer),
            Field::new("last_modified", ValueType::String),
        ]))
    }

    fn collect_entries(
        dir: &std::path::Path,
        pattern: Option<&regex::Regex>,
        recurse: bool,
        schema: &Arc<RowSchema>,
        rows: &mut Vec<Row>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;

            if meta.is_dir() {
                if recurse {
                    Self::collect_entries(&path, pattern, recurse, schema, rows)?;
                }
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_owned();

            if let Some(re) = pattern
                && !re.is_match(&filename)
            {
                continue;
            }

            let filepath = path.to_string_lossy().into_owned();
            let filesize = meta.len() as i64;
            let last_modified = meta
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs().to_string())
                })
                .unwrap_or_default();

            rows.push(Row::new(
                schema.clone(),
                vec![
                    Value::Str(filename),
                    Value::Str(filepath),
                    Value::Int(filesize),
                    Value::Str(last_modified),
                ],
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl Transform for GetFileNames {
    fn name(&self) -> &str {
        "GetFileNames"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok((*Self::schema()).clone())
    }

    fn is_source(&self) -> bool {
        true
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_dir = Some(
            resolve_context_path(ctx, &ctx.resolve(&self.config.directory))?
                .display()
                .to_string(),
        );
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn produce(&mut self, sender: mpsc::Sender<Row>) -> Result<()> {
        let dir_str = self
            .resolved_dir
            .as_deref()
            .unwrap_or(&self.config.directory);
        let dir = resolve_safe_path(dir_str)?;

        let pattern = self
            .config
            .pattern
            .as_deref()
            .map(|p| {
                regex::Regex::new(p)
                    .map_err(|e| AjisaiError::Config(format!("Invalid pattern: {}", e)))
            })
            .transpose()?;

        let schema = Self::schema();
        let mut rows = Vec::new();
        Self::collect_entries(
            &dir,
            pattern.as_ref(),
            self.config.include_subdirs,
            &schema,
            &mut rows,
        )
        .map_err(AjisaiError::Io)?;

        for row in rows {
            sender
                .send(row)
                .await
                .map_err(|_| AjisaiError::Pipeline("Downstream closed".into()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn lists_files_in_directory() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.csv"), "data").unwrap();
        fs::write(dir.path().join("b.txt"), "data").unwrap();

        let mut t = GetFileNames::new(GetFileNamesConfig {
            directory: dir.path().to_string_lossy().into_owned(),
            pattern: None,
            include_subdirs: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let (tx, mut rx) = mpsc::channel(16);
        t.produce(tx).await.unwrap();

        let mut names = Vec::new();
        while let Ok(row) = rx.try_recv() {
            if let Some(Value::Str(n)) = row.get("filename") {
                names.push(n.clone());
            }
        }
        names.sort();
        assert_eq!(names, vec!["a.csv", "b.txt"]);
    }

    #[tokio::test]
    async fn filters_by_pattern() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.csv"), "").unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();

        let mut t = GetFileNames::new(GetFileNamesConfig {
            directory: dir.path().to_string_lossy().into_owned(),
            pattern: Some(r"\.csv$".into()),
            include_subdirs: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let (tx, mut rx) = mpsc::channel(16);
        t.produce(tx).await.unwrap();

        let mut names = Vec::new();
        while let Ok(row) = rx.try_recv() {
            if let Some(Value::Str(n)) = row.get("filename") {
                names.push(n.clone());
            }
        }
        assert_eq!(names, vec!["a.csv"]);
    }
}
