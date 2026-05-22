use crate::model::hop_workflow::HopWorkflow;
use ajisai_core::{
    workflow::{Action, ActionResult, HopEvaluation, Workflow},
    AjisaiError, ExecutionContext, TransformRegistry,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

type RegistryFactory = Arc<dyn Fn() -> TransformRegistry + Send + Sync>;

// ── PipelineAction ────────────────────────────────────────────────────────────

pub struct PipelineAction {
    pub name: String,
    pub pipeline_path: PathBuf,
    registry_factory: RegistryFactory,
}

#[async_trait::async_trait]
impl Action for PipelineAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let path_str = ctx.resolve(self.pipeline_path.to_str().unwrap_or(""));
        let path = Path::new(&path_str);

        let hop_pipeline = match ajisai_hop_compat_inner::load_pipeline_file(path) {
            Ok(p) => p,
            Err(e) => return ActionResult::err(e.to_string()),
        };

        let registry = (self.registry_factory)();
        let pipeline =
            match crate::parser::pipeline::hop_pipeline_to_ajisai(hop_pipeline, &registry) {
                Ok(p) => p,
                Err(e) => return ActionResult::err(e.to_string()),
            };

        let engine = ajisai_core::PipelineEngine::new(pipeline, ctx.clone());
        match engine.run().await {
            Ok(_) => ActionResult::ok(),
            Err(e) => ActionResult::err(e.to_string()),
        }
    }
}

mod ajisai_hop_compat_inner {
    use crate::model::hop_pipeline::HopPipeline;
    use ajisai_core::AjisaiError;

    pub fn load_pipeline_file(path: &std::path::Path) -> Result<HopPipeline, AjisaiError> {
        let xml = std::fs::read_to_string(path).map_err(AjisaiError::Io)?;
        crate::parser::pipeline::parse_hpl(&xml)
    }
}

// ── ShellAction ───────────────────────────────────────────────────────────────

pub struct ShellAction {
    pub name: String,
    pub script: String,
    pub work_dir: Option<String>,
}

#[async_trait::async_trait]
impl Action for ShellAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let script = ctx.resolve(&self.script);
        let work_dir = self.work_dir.as_ref().map(|d| ctx.resolve(d));

        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = if cfg!(target_os = "windows") {
                let mut c = std::process::Command::new("cmd");
                c.args(["/C", &script]);
                c
            } else {
                let mut c = std::process::Command::new("sh");
                c.args(["-c", &script]);
                c
            };
            if let Some(dir) = work_dir {
                cmd.current_dir(dir);
            }
            cmd.output()
        })
        .await;

        match result {
            Ok(Ok(output)) if output.status.success() => ActionResult::ok(),
            Ok(Ok(output)) => {
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                ActionResult::err(if stderr.is_empty() {
                    format!("Shell exited with status {}", output.status)
                } else {
                    stderr
                })
            }
            Ok(Err(e)) => ActionResult::err(e.to_string()),
            Err(e) => ActionResult::err(format!("spawn_blocking: {}", e)),
        }
    }
}

// ── FileExistsAction ──────────────────────────────────────────────────────────

pub struct FileExistsAction {
    pub name: String,
    pub filename: String,
}

#[async_trait::async_trait]
impl Action for FileExistsAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let filename = ctx.resolve(&self.filename);
        match tokio::fs::metadata(&filename).await {
            Ok(m) if m.is_file() => ActionResult::ok(),
            Ok(_) => ActionResult::err(format!("'{}' exists but is not a file", filename)),
            Err(_) => ActionResult::err(format!("File '{}' does not exist", filename)),
        }
    }
}

// ── CreateFolderAction ────────────────────────────────────────────────────────

pub struct CreateFolderAction {
    pub name: String,
    pub foldername: String,
}

#[async_trait::async_trait]
impl Action for CreateFolderAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let folder = ctx.resolve(&self.foldername);
        match tokio::fs::create_dir_all(&folder).await {
            Ok(_) => ActionResult::ok(),
            Err(e) => ActionResult::err(e.to_string()),
        }
    }
}

// ── CopyFilesAction ───────────────────────────────────────────────────────────

pub struct CopyFilesAction {
    pub name: String,
    pub source: String,
    pub target: String,
}

#[async_trait::async_trait]
impl Action for CopyFilesAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let src = ctx.resolve(&self.source);
        let tgt = ctx.resolve(&self.target);
        match tokio::fs::copy(&src, &tgt).await {
            Ok(_) => ActionResult::ok(),
            Err(e) => ActionResult::err(e.to_string()),
        }
    }
}

// ── DeleteFilesAction ─────────────────────────────────────────────────────────

pub struct DeleteFilesAction {
    pub name: String,
    pub filename: String,
}

#[async_trait::async_trait]
impl Action for DeleteFilesAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let filename = ctx.resolve(&self.filename);
        match tokio::fs::remove_file(&filename).await {
            Ok(_) => ActionResult::ok(),
            Err(e) => ActionResult::err(e.to_string()),
        }
    }
}

// ── HttpWorkflowAction ────────────────────────────────────────────────────────

pub struct HttpWorkflowAction {
    pub name: String,
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
}

#[async_trait::async_trait]
impl Action for HttpWorkflowAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let url = ctx.resolve(&self.url);
        let client = reqwest::Client::new();

        let mut req_builder = match self.method.to_uppercase().as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };

        for (k, v) in &self.headers {
            req_builder = req_builder.header(k.as_str(), ctx.resolve(v));
        }

        if let Some(body) = &self.body {
            req_builder = req_builder.body(ctx.resolve(body));
        }

        match req_builder.send().await {
            Ok(resp) if resp.status().is_success() => ActionResult::ok(),
            Ok(resp) => ActionResult::err(format!("HTTP {} from {}", resp.status(), url)),
            Err(e) => ActionResult::err(e.to_string()),
        }
    }
}

// ── SetVariablesAction ────────────────────────────────────────────────────────

pub struct SetVariablesAction {
    pub name: String,
    /// (variable_name, value) pairs; values may contain ${VAR} substitutions
    pub variables: Vec<(String, String)>,
}

#[async_trait::async_trait]
impl Action for SetVariablesAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        for (k, v) in &self.variables {
            let resolved = ctx.resolve(v);
            // Set as OS environment variable so subsequent actions/pipelines see it
            std::env::set_var(k, &resolved);
        }
        ActionResult::ok()
    }
}

// ── hop_workflow_to_ajisai ────────────────────────────────────────────────────

/// Convert a HopWorkflow IR into an ajisai-core Workflow.
/// `base_dir` is used to resolve relative pipeline file paths.
/// `registry_factory` is called once per pipeline action to create a fresh registry.
pub fn hop_workflow_to_ajisai(
    hw: HopWorkflow,
    base_dir: &Path,
    registry_factory: impl Fn() -> TransformRegistry + Send + Sync + 'static,
) -> Result<Workflow, AjisaiError> {
    let factory: RegistryFactory = Arc::new(registry_factory);
    let mut workflow = Workflow::new(hw.name);

    for action in hw.actions {
        let action_name = action.name.clone();
        let type_key = action.type_name.to_uppercase();

        let act: Box<dyn Action> = match type_key.as_str() {
            "SHELL" => Box::new(ShellAction {
                name: action.name.clone(),
                script: action
                    .attributes
                    .get("script")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                work_dir: action
                    .attributes
                    .get("work_dir")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            }),

            "FILE_EXISTS" | "FILEEXISTS" => Box::new(FileExistsAction {
                name: action.name.clone(),
                filename: action
                    .attributes
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),

            "CREATE_FOLDER" | "CREATEFOLDER" => Box::new(CreateFolderAction {
                name: action.name.clone(),
                foldername: action
                    .attributes
                    .get("foldername")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),

            "COPY_FILES" | "COPYFILES" => Box::new(CopyFilesAction {
                name: action.name.clone(),
                source: action
                    .attributes
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                target: action
                    .attributes
                    .get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),

            "DELETE_FILES" | "DELETEFILES" => Box::new(DeleteFilesAction {
                name: action.name.clone(),
                filename: action
                    .attributes
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),

            "SET_VARIABLES" | "SETVARIABLES" => {
                // Expect attributes like "VAR_NAME" = "value" or a "variables" JSON array
                let mut variables: Vec<(String, String)> = Vec::new();
                if let Some(arr) = action.attributes.get("variables").and_then(|v| v.as_array()) {
                    for entry in arr {
                        if let (Some(k), Some(v)) = (
                            entry.get("name").and_then(|v| v.as_str()),
                            entry.get("value").and_then(|v| v.as_str()),
                        ) {
                            variables.push((k.to_string(), v.to_string()));
                        }
                    }
                } else {
                    // Fall back: treat all non-reserved attributes as variable=value
                    for (k, v) in &action.attributes {
                        if let Some(val) = v.as_str() {
                            variables.push((k.clone(), val.to_string()));
                        }
                    }
                }
                Box::new(SetVariablesAction { name: action.name.clone(), variables })
            }

            "HTTP" => Box::new(HttpWorkflowAction {
                name: action.name.clone(),
                url: action
                    .attributes
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                method: action
                    .attributes
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET")
                    .to_string(),
                body: action
                    .attributes
                    .get("body")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                headers: vec![],
            }),

            // Default: treat as pipeline execution
            _ => {
                let pipeline_file = action
                    .attributes
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let path = if pipeline_file.is_empty() {
                    base_dir.join(format!("{}.hpl", action.name))
                } else {
                    let p = Path::new(pipeline_file);
                    if p.is_absolute() {
                        p.to_owned()
                    } else {
                        base_dir.join(p)
                    }
                };

                Box::new(PipelineAction {
                    name: action.name.clone(),
                    pipeline_path: path,
                    registry_factory: Arc::clone(&factory),
                })
            }
        };

        workflow.add_action(action_name, act);
    }

    for hop in hw.hops {
        let evaluation = if hop.unconditional.unwrap_or(false) {
            HopEvaluation::Unconditional
        } else {
            match hop.evaluation.as_deref() {
                Some("true") => HopEvaluation::Success,
                Some("false") => HopEvaluation::Failure,
                _ => HopEvaluation::Unconditional,
            }
        };
        workflow.add_hop(
            hop.from,
            hop.to,
            evaluation,
            hop.unconditional.unwrap_or(false),
        );
    }

    Ok(workflow)
}
