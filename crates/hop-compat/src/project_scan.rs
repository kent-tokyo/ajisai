use crate::{
    HopCompatibilityReport, WorkflowAssessment, assess_pipeline, assess_workflow,
    load_pipeline_file, load_workflow_file,
};
use ajisai_core::{AjisaiError, TransformRegistry};
use serde::Serialize;
use std::path::Path;

const MAX_PROJECT_FILE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectFile {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectScanReport {
    pub schema_version: u16,
    pub format: &'static str,
    pub root: String,
    pub files: Vec<ProjectFile>,
    pub pipelines: Vec<HopCompatibilityReport>,
    pub workflows: Vec<WorkflowInventory>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowInventory {
    pub name: String,
    pub actions: Vec<WorkflowAssessment>,
    pub hops: usize,
}

/// Inventory a Hop project without executing files or following symlinks.
pub fn scan_project(
    root: &Path,
    registry: &TransformRegistry,
) -> Result<ProjectScanReport, AjisaiError> {
    if !root.is_dir() {
        return Err(AjisaiError::Config(format!(
            "Project root is not a directory: {}",
            root.display()
        )));
    }
    let mut files = Vec::new();
    let mut pipelines = Vec::new();
    let mut workflows = Vec::new();
    let mut warnings = Vec::new();
    visit(
        root,
        root,
        registry,
        &mut files,
        &mut pipelines,
        &mut workflows,
        &mut warnings,
    )?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    pipelines.sort_by(|left, right| left.pipeline.cmp(&right.pipeline));
    workflows.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(ProjectScanReport {
        schema_version: 1,
        format: "hop.project-scan",
        root: root.display().to_string(),
        files,
        pipelines,
        workflows,
        warnings,
    })
}

fn visit(
    root: &Path,
    directory: &Path,
    registry: &TransformRegistry,
    files: &mut Vec<ProjectFile>,
    pipelines: &mut Vec<HopCompatibilityReport>,
    workflows: &mut Vec<WorkflowInventory>,
    warnings: &mut Vec<String>,
) -> Result<(), AjisaiError> {
    for entry in std::fs::read_dir(directory).map_err(AjisaiError::Io)? {
        let entry = entry.map_err(AjisaiError::Io)?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(AjisaiError::Io)?;
        if metadata.file_type().is_symlink() {
            warnings.push(format!("SYMLINK_SKIPPED:{}", relative(root, &path)));
            continue;
        }
        if metadata.is_dir() {
            visit(root, &path, registry, files, pipelines, workflows, warnings)?;
            continue;
        }
        let bytes = metadata.len();
        let relative_path = relative(root, &path);
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let kind = match extension {
            "hpl" => "pipeline",
            "hwf" => "workflow",
            "json" | "yaml" | "yml" | "properties" | "xml" => "metadata-or-dependency",
            _ => "other",
        };
        files.push(ProjectFile {
            path: relative_path.clone(),
            kind: kind.into(),
            bytes,
        });
        if bytes > MAX_PROJECT_FILE_BYTES {
            warnings.push(format!("OVERSIZED_FILE:{}", relative_path));
            continue;
        }
        if extension == "hpl" || extension == "hwf" {
            let source = std::fs::read(&path).map_err(AjisaiError::Io)?;
            if contains_unsafe_xml(&source) {
                warnings.push(format!("UNSAFE_XML_CONSTRUCT:{}", relative_path));
                continue;
            }
            if extension == "hpl" {
                match load_pipeline_file(&path) {
                    Ok(pipeline) => pipelines.push(assess_pipeline(&pipeline, registry)),
                    Err(error) => {
                        warnings.push(format!("PIPELINE_PARSE_ERROR:{}:{}", relative_path, error))
                    }
                }
            } else {
                match load_workflow_file(&path) {
                    Ok(workflow) => workflows.push(WorkflowInventory {
                        name: workflow.name.clone(),
                        actions: assess_workflow(&workflow),
                        hops: workflow.hops.len(),
                    }),
                    Err(error) => {
                        warnings.push(format!("WORKFLOW_PARSE_ERROR:{}:{}", relative_path, error))
                    }
                }
            }
        }
    }
    Ok(())
}

fn contains_unsafe_xml(source: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(source).to_ascii_lowercase();
    [
        "<!doctype",
        "<!entity",
        "<script",
        "<?xml-stylesheet",
        "xsi:type=\"script",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::TransformRegistry;
    use std::sync::Arc;
    #[test]
    fn scans_pipelines_and_skips_symlinks() {
        let root = std::env::temp_dir().join(format!("ajisai-project-scan-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("a.hpl"), "<pipeline><name>A</name></pipeline>").unwrap();
        std::fs::write(root.join("notes.txt"), "external").unwrap();
        std::fs::write(root.join("flow.hwf"), "<workflow><name>Flow</name><action><name>run</name><type>Pipeline</type></action></workflow>").unwrap();
        let link = root.join("link.hpl");
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("a.hpl"), &link).unwrap();
        let mut registry = TransformRegistry::new();
        registry.register(
            "CsvFileInput",
            Arc::new(|_| Err(AjisaiError::Config("test".into()))),
        );
        let report = scan_project(&root, &registry).unwrap();
        assert_eq!(report.pipelines.len(), 1);
        assert_eq!(report.workflows.len(), 1);
        assert_eq!(
            report.workflows[0].actions[0].reason_code,
            "WORKFLOW_ACTION_AS_PIPELINE"
        );
        assert!(report.files.iter().any(|file| file.path == "notes.txt"));
        #[cfg(unix)]
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.starts_with("SYMLINK_SKIPPED:"))
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_active_xml_before_parsing() {
        let root =
            std::env::temp_dir().join(format!("ajisai-project-scan-unsafe-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("unsafe.hpl"),
            "<!DOCTYPE pipeline [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><pipeline/>",
        )
        .unwrap();
        let registry = TransformRegistry::new();
        let report = scan_project(&root, &registry).unwrap();
        assert!(report.pipelines.is_empty());
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.starts_with("UNSAFE_XML_CONSTRUCT:"))
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
