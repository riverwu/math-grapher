//! User Interface Module
//!
//! Provides the main application UI using egui.

mod app;
mod expr_panel;
mod graph_view;
mod history;
mod math_display;
mod project;
mod slider;
mod syntax;
mod toolbar;
mod settings;

pub use app::MathGrapherApp;
pub use math_display::MathFormatter;
pub use expr_panel::{ExpressionPanel, ExpressionEntry};
pub use graph_view::GraphView;
pub use history::{History, Action};
pub use project::{ProjectData, ExpressionData, ParameterData};
pub use slider::{ParameterSlider, SliderConfig};
pub use syntax::{SyntaxHighlighter, syntax_highlighted_text_edit};
pub use toolbar::Toolbar;
pub use settings::SettingsPanel;
