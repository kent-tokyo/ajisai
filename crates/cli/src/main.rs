mod commands;

rust_i18n::i18n!("locales", fallback = "en");

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

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
    /// Execute a pipeline (.hpl file)
    Run {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        #[arg(short = 'e', long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,
    },

    /// Validate a pipeline without executing it
    Validate {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
    },

    /// Execute a workflow (.hwf file)
    #[command(name = "run-workflow")]
    RunWorkflow {
        #[arg(short = 'p', long)]
        pipeline: PathBuf,
        #[arg(short = 'e', long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,
    },

    /// List all available transform types
    #[command(name = "list-transforms")]
    ListTransforms,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    rust_i18n::set_locale(&cli.lang);

    let filter = EnvFilter::try_new(&cli.log_level).unwrap_or_else(|_| EnvFilter::new("warn"));
    fmt().with_env_filter(filter).init();

    let result = match cli.command {
        Commands::Run { pipeline, env } => commands::run::run(pipeline, env).await,
        Commands::Validate { pipeline } => {
            commands::validate::validate(pipeline).map_err(Into::into)
        }
        Commands::RunWorkflow { pipeline, env } => {
            commands::run_workflow::run_workflow(pipeline, env).await
        }
        Commands::ListTransforms => {
            commands::list::list_transforms();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
