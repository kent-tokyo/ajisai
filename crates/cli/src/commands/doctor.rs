use serde_json::json;
use std::path::PathBuf;

fn user_dir(env_name: &str, windows_env_name: &str, fallback_suffix: &str) -> Option<PathBuf> {
    std::env::var_os(env_name)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(windows_env_name).map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback_suffix)))
}

pub fn doctor(json_output: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let temp = std::env::temp_dir();
    let config_dir =
        user_dir("XDG_CONFIG_HOME", "APPDATA", ".config").map(|path| path.join("ajisai"));
    let cache_dir =
        user_dir("XDG_CACHE_HOME", "LOCALAPPDATA", ".cache").map(|path| path.join("ajisai"));
    let report = json!({
        "schema_version": 1,
        "cwd": cwd,
        "temp_dir": temp,
        "config_dir": config_dir,
        "cache_dir": cache_dir,
        "cwd_exists": cwd.is_dir(),
        "temp_dir_exists": temp.is_dir(),
        "network_default": "allowed",
        "policies": [
            "--max-rows",
            "--max-buffered-rows",
            "--timeout-secs",
            "--no-network",
            "--project-root"
        ],
        "native_schemas": ["spec/ajisai.pipeline.schema.json", "spec/ajisai.workflow.schema.json"]
    });
    if json_output {
        println!("{}", report);
    } else {
        println!("Ajisai doctor (schema_version=1)");
        println!("  cwd: {}", cwd.display());
        println!("  temp_dir: {}", temp.display());
        println!(
            "  config_dir: {}",
            config_dir
                .as_deref()
                .map_or_else(|| "unavailable".into(), |path| path.display().to_string())
        );
        println!(
            "  cache_dir: {}",
            cache_dir
                .as_deref()
                .map_or_else(|| "unavailable".into(), |path| path.display().to_string())
        );
        println!("  cwd_exists: {}", cwd.is_dir());
        println!("  temp_dir_exists: {}", temp.is_dir());
        println!("  network_default: allowed (use --no-network to deny)");
        println!(
            "  resource policies: --max-rows, --max-buffered-rows, --timeout-secs, --no-network, --project-root"
        );
    }
    Ok(())
}
