mod commands;
mod spinner;

// Locale files are embedded at compile time by rust-i18n.

/// Stable process exit codes for automation clients.
const EXIT_RUNTIME_ERROR: i32 = 1;
const EXIT_NOT_FOUND: i32 = 2;
const EXIT_VALIDATION: i32 = 3;
const EXIT_CANCELLED: i32 = 130;

rust_i18n::i18n!("locales", fallback = "en");

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser)]
#[command(
    name = "ajisai-cli",
    about = "Ajisai ETL — Apache Hop compatible pipeline runner",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, global = true, default_value = "warn")]
    log_level: String,

    /// UI language: en, ja
    #[arg(long, global = true, default_value = "en", value_name = "LANG")]
    lang: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Assess Hop transform compatibility without executing the pipeline
    Assess {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        /// Emit a machine-readable compatibility report
        #[arg(long)]
        json: bool,
    },
    /// Diagnose local capabilities and configured safety policies
    Doctor {
        /// Emit a stable JSON report
        #[arg(long)]
        json: bool,
    },
    /// Explain a native pipeline/workflow without executing it
    Explain {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
    },
    /// Validate and rewrite a native document using the current schema
    Migrate {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    /// Create a minimal native pipeline document
    New {
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(short = 'n', long)]
        name: Option<String>,
    },
    /// Inspect a native pipeline/workflow without executing it
    Inspect {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
    },
    /// Preview bounded rows from a native CSV source without running sinks
    Preview {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        #[arg(short = 'n', long, default_value_t = 20)]
        rows: usize,
    },
    /// Execute a pipeline (.hpl file)
    Run {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        #[arg(short = 'e', long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,
        /// Maximum number of input rows accepted before failing safely
        #[arg(long)]
        max_rows: Option<u64>,
        /// Maximum number of rows retained by blocking transforms
        #[arg(long)]
        max_buffered_rows: Option<u64>,
        /// Maximum execution time in seconds before failing safely
        #[arg(long)]
        timeout_secs: Option<u64>,
        /// Disable network-capable transforms for this run
        #[arg(long)]
        no_network: bool,
        /// Emit one stable JSON result envelope on success
        #[arg(long)]
        json: bool,
        /// Build and validate the execution plan without running transforms
        #[arg(long)]
        dry_run: bool,
        /// Persist the JSON run or dry-run record atomically at this path
        #[arg(long, value_name = "PATH")]
        record: Option<PathBuf>,
        /// Restrict filesystem transforms to this project root
        #[arg(long, value_name = "PATH")]
        project_root: Option<PathBuf>,
    },

    /// Validate a pipeline without executing it
    Validate {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        /// Emit one stable JSON object instead of human-readable output
        #[arg(long)]
        json: bool,
    },

    /// Execute a workflow (.hwf file)
    #[command(name = "run-workflow")]
    RunWorkflow {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        #[arg(short = 'e', long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,
        /// Restrict filesystem transforms in workflow actions to this root
        #[arg(long, value_name = "PATH")]
        project_root: Option<PathBuf>,
        /// Emit one versioned JSON workflow result envelope
        #[arg(long)]
        json: bool,
        /// Validate and summarize the workflow without executing actions
        #[arg(long)]
        dry_run: bool,
        /// Persist the workflow result envelope atomically at this path
        #[arg(long, value_name = "PATH")]
        record: Option<PathBuf>,
    },
    /// Inventory a Hop project without executing it
    Scan {
        #[arg(short = 'p', long)]
        project: PathBuf,
        /// Emit a machine-readable project report
        #[arg(long)]
        json: bool,
    },

    /// List all available transform types
    #[command(name = "list-transforms")]
    ListTransforms {
        /// Emit versioned transform manifests as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate shell completion scripts from the CLI contract
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let json_run = matches!(
        &cli.command,
        Commands::Run { json: true, .. } | Commands::RunWorkflow { json: true, .. }
    );
    let workflow_json = matches!(&cli.command, Commands::RunWorkflow { json: true, .. });
    let record_path = match &cli.command {
        Commands::Run { record, .. } => record.clone(),
        Commands::RunWorkflow { record, .. } => record.clone(),
        _ => None,
    };
    let run_id = format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default(),
        std::process::id()
    );

    rust_i18n::set_locale(&cli.lang);

    let filter = EnvFilter::try_new(&cli.log_level).unwrap_or_else(|_| EnvFilter::new("warn"));
    fmt().with_env_filter(filter).init();

    let result = match cli.command {
        Commands::Assess { pipeline, json } => commands::assess::assess(pipeline, json),
        Commands::Doctor { json } => commands::doctor::doctor(json),
        Commands::Migrate { pipeline, output } => commands::migrate::migrate(pipeline, output),
        Commands::New { output, name } => commands::new::new_pipeline(output, name),
        Commands::Explain { pipeline } => commands::explain::explain(pipeline),
        Commands::Inspect { pipeline } => commands::inspect::inspect(pipeline),
        Commands::Preview { pipeline, rows } => commands::preview::preview(pipeline, rows).await,
        Commands::Run {
            pipeline,
            env,
            max_rows,
            max_buffered_rows,
            timeout_secs,
            no_network,
            json,
            dry_run,
            record,
            project_root,
        } => {
            commands::run::run(
                pipeline,
                env,
                commands::run::RunOptions {
                    max_rows,
                    max_buffered_rows,
                    timeout_secs,
                    no_network,
                    json_output: json,
                    dry_run,
                    record_path: record,
                    run_id: run_id.clone(),
                    project_root,
                },
            )
            .await
        }
        Commands::Validate { pipeline, json } => commands::validate::validate(pipeline, json),
        Commands::RunWorkflow {
            pipeline,
            env,
            project_root,
            json,
            dry_run,
            record,
        } => {
            commands::run_workflow::run_workflow(
                pipeline,
                env,
                project_root,
                json,
                dry_run,
                record,
                run_id.clone(),
            )
            .await
        }
        Commands::Scan { project, json } => commands::scan::scan(project, json),
        Commands::ListTransforms { json } => {
            commands::list::list_transforms(json);
            Ok(())
        }
        Commands::Completions { shell } => {
            let mut command = Cli::command();
            generate(shell, &mut command, "ajisai-cli", &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(e) = result {
        let exit_code = stable_exit_code(&e);
        let envelope = serde_json::json!({
            "schema_version": 1,
            "status": "failed",
            "run_id": run_id,
            "error": e.to_string(),
            "exit_code": exit_code,
        });
        if json_run {
            println!("{}", envelope);
        } else {
            eprintln!("Error: {}", e);
        }
        let should_persist_fallback = record_path
            .as_deref()
            .is_some_and(|path| !workflow_json || !path.exists());
        if should_persist_fallback
            && let Some(path) = record_path.as_deref()
            && let Err(record_error) = commands::run::persist_record(Some(path), &envelope)
        {
            eprintln!("Could not persist run record: {record_error}");
        }
        std::process::exit(exit_code);
    }
}

fn stable_exit_code(error: &anyhow::Error) -> i32 {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("cancelled") {
        EXIT_CANCELLED
    } else if message.contains("not found")
        || message.contains("file_not_found")
        || message.contains("ファイルが見つかりません")
    {
        EXIT_NOT_FOUND
    } else if message.contains("validation")
        || message.contains("invalid")
        || message.contains("unsupported")
        || message.contains("検証")
        || message.contains("非対応")
    {
        EXIT_VALIDATION
    } else {
        EXIT_RUNTIME_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_exit_codes_classify_automation_failures() {
        assert_eq!(
            stable_exit_code(&anyhow::anyhow!("file not found")),
            EXIT_NOT_FOUND
        );
        assert_eq!(
            stable_exit_code(&anyhow::anyhow!("error.file_not_found")),
            EXIT_NOT_FOUND
        );
        assert_eq!(
            stable_exit_code(&anyhow::anyhow!("ファイルが見つかりません: /tmp/missing")),
            EXIT_NOT_FOUND
        );
        assert_eq!(
            stable_exit_code(&anyhow::anyhow!("native validation failed")),
            EXIT_VALIDATION
        );
        assert_eq!(
            stable_exit_code(&anyhow::anyhow!("Pipeline cancelled by user")),
            EXIT_CANCELLED
        );
        assert_eq!(
            stable_exit_code(&anyhow::anyhow!("database unavailable")),
            EXIT_RUNTIME_ERROR
        );
    }
}
