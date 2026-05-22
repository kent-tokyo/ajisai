mod app;
mod canvas;
mod config_form;
mod runner;
mod state;

rust_i18n::i18n!("locales", fallback = "en");

use app::AjisaiApp;
use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("warn"))
        .init();

    // Default to system locale if Japanese, otherwise English
    let lang = std::env::var("AJISAI_LANG").unwrap_or_else(|_| "en".into());
    rust_i18n::set_locale(&lang);

    // Parse --open <file> and --run arguments
    let args: Vec<String> = std::env::args().collect();
    let preload: Option<std::path::PathBuf> = args
        .windows(2)
        .find(|w| w[0] == "--open")
        .map(|w| std::path::PathBuf::from(&w[1]));
    let run_on_start = args.contains(&"--run".to_string());
    let select_first = args.contains(&"--select-first".to_string());

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Ajisai — Visual ETL")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Ajisai",
        options,
        Box::new(move |cc| {
            if let Some(path) = preload {
                if run_on_start {
                    Ok(Box::new(AjisaiApp::with_open_and_run(cc, path)))
                } else if select_first {
                    Ok(Box::new(AjisaiApp::with_open_select_first(cc, path)))
                } else {
                    Ok(Box::new(AjisaiApp::with_open(cc, path)))
                }
            } else {
                Ok(Box::new(AjisaiApp::new(cc)))
            }
        }),
    )
}
