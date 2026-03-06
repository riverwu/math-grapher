//! Math Grapher - A Desmos-like mathematical graphing calculator
//!
//! This application provides:
//! - Expression parsing and evaluation
//! - GPU-accelerated curve rendering
//! - Interactive graph manipulation (pan, zoom)
//! - Intersection and curve fitting calculations

use eframe::egui;
use math_grapher::ui::MathGrapherApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Math Grapher"),
        ..Default::default()
    };

    eframe::run_native(
        "Math Grapher",
        options,
        Box::new(|cc| Ok(Box::new(MathGrapherApp::new(cc)))),
    )
}
