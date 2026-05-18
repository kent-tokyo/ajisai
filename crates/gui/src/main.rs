mod app;
mod canvas;
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
        Box::new(|cc| Ok(Box::new(AjisaiApp::new(cc)))),
    )
}
