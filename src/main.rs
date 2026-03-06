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
        Box::new(|cc| {
            // Configure fonts for better math display
            let fonts = egui::FontDefinitions::default();

            // Configure font families priority
            // The default proportional font is good for UI
            // We can add custom math fonts here if needed in the future

            // Tweak text sizes slightly for better readability
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(14.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(14.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(18.0),
            );
            cc.egui_ctx.set_style(style);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(MathGrapherApp::new(cc)))
        }),
    )
}
