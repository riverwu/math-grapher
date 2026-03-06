//! Main application

use eframe::egui;
use std::f64::consts::PI;
use crate::common::{Color, Point, Rect, CURVE_COLORS};
use crate::parser::{parse_full_equation, ParsedEquation, ExpressionType, AstNode, ComparisonOp};
use crate::evaluator::{
    evaluate_explicit, evaluate_implicit, evaluate_parametric, evaluate_polar,
    evaluate_inequality, CurveData, InequalityRegion,
};
use crate::algebra::{find_intersections, CurveFitter, FitModel, FitResult};
use crate::render::{
    GraphCanvas, CoordinateTransform, CurveRenderer, CurveStyle,
    MarkerRenderer, Marker, MarkerType, RenderContext, CanvasInteraction,
    RegionRenderer, RegionStyle,
};

use super::{ExpressionPanel, GraphView, SettingsPanel, ParameterSlider};

/// A compiled expression ready for rendering
#[derive(Debug, Clone)]
pub struct CompiledExpression {
    /// Original expression string
    pub source: String,
    /// Parsed AST (for explicit, implicit, polar)
    pub ast: AstNode,
    /// Expression type
    pub expr_type: ExpressionType,
    /// Display color
    pub color: Color,
    /// Is visible
    pub visible: bool,
    /// Cached curve data (for explicit, parametric, polar)
    pub curve_data: Option<CurveData>,
    /// Cached implicit segments
    pub implicit_segments: Option<Vec<(Point, Point)>>,
    /// Parametric ASTs (x(t), y(t))
    pub parametric_ast: Option<(AstNode, AstNode)>,
    /// Inequality region data
    pub inequality_region: Option<InequalityRegion>,
    /// Inequality comparison operator
    pub inequality_op: Option<ComparisonOp>,
}

impl CompiledExpression {
    pub fn new(source: String, ast: AstNode, expr_type: ExpressionType, color: Color) -> Self {
        Self {
            source,
            ast,
            expr_type,
            color,
            visible: true,
            curve_data: None,
            implicit_segments: None,
            parametric_ast: None,
            inequality_region: None,
            inequality_op: None,
        }
    }

    /// Create a new parametric expression
    pub fn new_parametric(
        source: String,
        x_ast: AstNode,
        y_ast: AstNode,
        color: Color,
    ) -> Self {
        Self {
            source,
            ast: AstNode::Number(0.0), // Placeholder
            expr_type: ExpressionType::Parametric,
            color,
            visible: true,
            curve_data: None,
            implicit_segments: None,
            parametric_ast: Some((x_ast, y_ast)),
            inequality_region: None,
            inequality_op: None,
        }
    }

    /// Create a new inequality expression
    pub fn new_inequality(
        source: String,
        ast: AstNode,
        op: ComparisonOp,
        color: Color,
    ) -> Self {
        Self {
            source,
            ast,
            expr_type: ExpressionType::Inequality,
            color,
            visible: true,
            curve_data: None,
            implicit_segments: None,
            parametric_ast: None,
            inequality_region: None,
            inequality_op: Some(op),
        }
    }

    /// Update the curve cache for current viewport
    pub fn update_cache(&mut self, viewport: &Rect) {
        match self.expr_type {
            ExpressionType::Explicit => {
                match evaluate_explicit(&self.ast, viewport, 500) {
                    Ok(data) => self.curve_data = Some(data),
                    Err(_) => self.curve_data = None,
                }
            }
            ExpressionType::Implicit => {
                match evaluate_implicit(&self.ast, viewport, 100) {
                    Ok(segments) => self.implicit_segments = Some(segments),
                    Err(_) => self.implicit_segments = None,
                }
            }
            ExpressionType::Parametric => {
                if let Some((ref x_ast, ref y_ast)) = self.parametric_ast {
                    // Default parameter range: 0 to 2*PI
                    match evaluate_parametric(x_ast, y_ast, (0.0, 2.0 * PI), 500) {
                        Ok(data) => self.curve_data = Some(data),
                        Err(_) => self.curve_data = None,
                    }
                }
            }
            ExpressionType::Polar => {
                // Default theta range: 0 to 2*PI
                match evaluate_polar(&self.ast, (0.0, 2.0 * PI), 500) {
                    Ok(data) => self.curve_data = Some(data),
                    Err(_) => self.curve_data = None,
                }
            }
            ExpressionType::Inequality => {
                if let Some(op) = self.inequality_op {
                    match evaluate_inequality(&self.ast, op, viewport, 80) {
                        Ok(region) => self.inequality_region = Some(region),
                        Err(_) => self.inequality_region = None,
                    }
                }
            }
        }
    }
}

/// Main application state
pub struct MathGrapherApp {
    /// Expression panel state
    expression_panel: ExpressionPanel,
    /// Graph view state
    graph_view: GraphView,
    /// Settings panel state
    settings_panel: SettingsPanel,
    /// Compiled expressions
    expressions: Vec<CompiledExpression>,
    /// Markers (intersections, etc.)
    markers: Vec<Marker>,
    /// Parameter sliders
    sliders: Vec<ParameterSlider>,
    /// Show settings panel
    show_settings: bool,
    /// Status message
    status_message: Option<String>,
    /// Graph canvas
    canvas: GraphCanvas,
    /// Canvas interaction handler
    interaction: CanvasInteraction,
    /// Curve renderer
    curve_renderer: CurveRenderer,
    /// Marker renderer
    marker_renderer: MarkerRenderer,
    /// Region renderer (for inequalities)
    region_renderer: RegionRenderer,
    /// Need to recalculate curves
    needs_recalc: bool,

    // Curve fitting state
    /// Data points for curve fitting
    fit_data_points: Vec<Point>,
    /// Selected fit model
    fit_model: FitModel,
    /// Current fit result
    fit_result: Option<FitResult>,
    /// Is fitting mode active (click to add points)
    fitting_mode: bool,
    /// Show fitting panel
    show_fitting_panel: bool,
}

impl MathGrapherApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            expression_panel: ExpressionPanel::new(),
            graph_view: GraphView::new(),
            settings_panel: SettingsPanel::new(),
            expressions: Vec::new(),
            markers: Vec::new(),
            sliders: Vec::new(),
            show_settings: false,
            status_message: None,
            canvas: GraphCanvas::new(),
            interaction: CanvasInteraction::new(),
            curve_renderer: CurveRenderer::new(),
            marker_renderer: MarkerRenderer::new(),
            region_renderer: RegionRenderer::new(),
            needs_recalc: false,
            // Curve fitting
            fit_data_points: Vec::new(),
            fit_model: FitModel::Linear,
            fit_result: None,
            fitting_mode: false,
            show_fitting_panel: false,
        }
    }

    /// Compile an expression and add it to the list
    fn add_expression(&mut self, source: &str) {
        match parse_full_equation(source) {
            Ok(parsed) => {
                let color = CURVE_COLORS[self.expressions.len() % CURVE_COLORS.len()];
                let mut expr = match parsed {
                    ParsedEquation::Explicit(ast) => {
                        CompiledExpression::new(source.to_string(), ast, ExpressionType::Explicit, color)
                    }
                    ParsedEquation::Implicit(ast) => {
                        CompiledExpression::new(source.to_string(), ast, ExpressionType::Implicit, color)
                    }
                    ParsedEquation::Polar(ast) => {
                        CompiledExpression::new(source.to_string(), ast, ExpressionType::Polar, color)
                    }
                    ParsedEquation::Parametric { x_ast, y_ast } => {
                        CompiledExpression::new_parametric(source.to_string(), x_ast, y_ast, color)
                    }
                    ParsedEquation::Inequality { expr, op } => {
                        CompiledExpression::new_inequality(source.to_string(), expr, op, color)
                    }
                };
                expr.update_cache(&self.canvas.viewport);
                self.expressions.push(expr);
                self.status_message = None;
            }
            Err(e) => {
                self.status_message = Some(format!("Parse error: {}", e));
            }
        }
    }

    /// Remove an expression by index
    fn remove_expression(&mut self, index: usize) {
        if index < self.expressions.len() {
            self.expressions.remove(index);
        }
    }

    /// Recalculate all curves for current viewport
    fn recalculate_curves(&mut self) {
        for expr in &mut self.expressions {
            if expr.visible {
                expr.update_cache(&self.canvas.viewport);
            }
        }
        self.update_intersections();
        self.needs_recalc = false;
    }

    /// Find and update intersection markers between all visible explicit curves
    fn update_intersections(&mut self) {
        // Keep data point markers, clear only intersection markers
        self.markers.retain(|m| m.marker_type == MarkerType::DataPoint);

        // Collect visible explicit functions for intersection detection
        let explicit_exprs: Vec<(usize, &AstNode)> = self.expressions
            .iter()
            .enumerate()
            .filter(|(_, e)| e.visible && e.expr_type == ExpressionType::Explicit)
            .map(|(i, e)| (i, &e.ast))
            .collect();

        // Find intersections between each pair
        let x_range = (self.canvas.viewport.x_min, self.canvas.viewport.x_max);
        let tolerance = self.canvas.viewport.width() / 1000.0;

        for i in 0..explicit_exprs.len() {
            for j in (i + 1)..explicit_exprs.len() {
                let (_, ast_i) = explicit_exprs[i];
                let (_, ast_j) = explicit_exprs[j];

                if let Ok(points) = find_intersections(ast_i, ast_j, x_range, tolerance) {
                    for point in points {
                        // Only add markers within viewport
                        if point.y >= self.canvas.viewport.y_min
                            && point.y <= self.canvas.viewport.y_max
                        {
                            self.markers.push(Marker::intersection(point));
                        }
                    }
                }
            }
        }
    }

    /// Update curve fit based on current data points
    fn update_fit(&mut self) {
        if self.fit_data_points.len() >= 2 {
            let fitter = CurveFitter::new();
            self.fit_result = fitter.fit(&self.fit_data_points, self.fit_model);
        } else {
            self.fit_result = None;
        }
    }

    /// Add fitted curve to expressions
    fn add_fit_to_expressions(&mut self) {
        if let Some(ref result) = self.fit_result {
            let expr_str = format!("y = {}", result.to_expression());
            self.add_expression(&expr_str);
        }
    }

    /// Clear fitting data
    fn clear_fit_data(&mut self) {
        self.fit_data_points.clear();
        self.fit_result = None;
        // Remove data point markers
        self.markers.retain(|m| m.marker_type != MarkerType::DataPoint);
    }

    /// Render the graph canvas
    fn render_canvas(&self, painter: &egui::Painter, rect: egui::Rect) {
        // Render grid and axes
        self.canvas.render(painter, rect);

        // Create transform and context
        let transform = CoordinateTransform::new(
            self.canvas.viewport,
            rect.width(),
            rect.height(),
        );
        let ctx = RenderContext::new(transform, painter, rect);

        // First pass: render inequality regions (below curves)
        for expr in &self.expressions {
            if !expr.visible {
                continue;
            }

            if expr.expr_type == ExpressionType::Inequality {
                if let Some(ref region) = expr.inequality_region {
                    let style = RegionStyle::from_color(expr.color);
                    self.region_renderer.render(&ctx, region, &style);
                }
            }
        }

        // Second pass: render curves
        for expr in &self.expressions {
            if !expr.visible {
                continue;
            }

            let style = CurveStyle::default().with_color(expr.color);

            match expr.expr_type {
                ExpressionType::Explicit | ExpressionType::Parametric | ExpressionType::Polar => {
                    if let Some(ref data) = expr.curve_data {
                        self.curve_renderer.render(&ctx, data, &style);
                    }
                }
                ExpressionType::Implicit => {
                    if let Some(ref segments) = expr.implicit_segments {
                        self.curve_renderer.render_implicit(&ctx, segments, &style);
                    }
                }
                ExpressionType::Inequality => {
                    // Boundary was already rendered with the region
                }
            }
        }

        // Render fitted curve if available
        if let Some(ref fit_result) = self.fit_result {
            let mut fit_curve = CurveData::with_capacity(200);
            let x_step = self.canvas.viewport.width() / 199.0;
            let mut prev_valid = false;

            for i in 0..200 {
                let x = self.canvas.viewport.x_min + i as f64 * x_step;
                let y = fit_result.evaluate(x);

                if y.is_finite() {
                    fit_curve.points.push(Point::new(x, y));
                    if prev_valid && fit_curve.points.len() > 1 {
                        fit_curve.continuous.push(true);
                    } else if fit_curve.points.len() > 1 {
                        fit_curve.continuous.push(false);
                    }
                    prev_valid = true;
                } else {
                    fit_curve.points.push(Point::new(f64::NAN, f64::NAN));
                    if fit_curve.points.len() > 1 {
                        fit_curve.continuous.push(false);
                    }
                    prev_valid = false;
                }
            }

            let fit_style = CurveStyle::default()
                .with_color(Color::rgb(0.9, 0.3, 0.1))
                .with_width(2.5);
            self.curve_renderer.render(&ctx, &fit_curve, &fit_style);
        }

        // Render data points for fitting
        self.marker_renderer.render_data_points(&ctx, &self.fit_data_points);

        // Render markers (intersections, etc.)
        self.marker_renderer.render_all(&ctx, &self.markers);
    }
}

impl eframe::App for MathGrapherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Math Grapher");
                ui.separator();

                if ui.button("Reset View").clicked() {
                    self.canvas.reset_viewport();
                    self.needs_recalc = true;
                }

                if ui.button("Zoom In").clicked() {
                    self.canvas.zoom(0.8, self.canvas.viewport.center());
                    self.needs_recalc = true;
                }

                if ui.button("Zoom Out").clicked() {
                    self.canvas.zoom(1.25, self.canvas.viewport.center());
                    self.needs_recalc = true;
                }

                ui.separator();

                if ui.button("Settings").clicked() {
                    self.show_settings = !self.show_settings;
                }

                let fit_button_text = if self.fitting_mode { "Exit Fit Mode" } else { "Curve Fit" };
                if ui.button(fit_button_text).clicked() {
                    self.show_fitting_panel = !self.show_fitting_panel;
                    if !self.show_fitting_panel {
                        self.fitting_mode = false;
                    }
                }

                if self.fitting_mode {
                    ui.label(egui::RichText::new("Click to add points").color(egui::Color32::YELLOW));
                }

                // Show mouse coordinates
                if let Some(pos) = self.interaction.mouse_world_pos {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("({:.3}, {:.3})", pos.x, pos.y));
                    });
                }
            });
        });

        // Left panel - expression list
        egui::SidePanel::left("expressions")
            .default_width(300.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Expressions");
                ui.separator();

                // Input field for new expression
                let mut new_expr = String::new();
                let response = ui.horizontal(|ui| {
                    let text_edit = ui.text_edit_singleline(&mut self.expression_panel.input_buffer);
                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        new_expr = self.expression_panel.input_buffer.clone();
                        self.expression_panel.input_buffer.clear();
                    }
                    if ui.button("Add").clicked() && !self.expression_panel.input_buffer.is_empty() {
                        new_expr = self.expression_panel.input_buffer.clone();
                        self.expression_panel.input_buffer.clear();
                    }
                });

                if !new_expr.is_empty() {
                    self.add_expression(&new_expr);
                }

                // Show status message
                if let Some(ref msg) = self.status_message {
                    ui.colored_label(egui::Color32::RED, msg);
                }

                ui.separator();

                // Expression list
                let mut to_remove = None;
                let mut visibility_changed = false;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, expr) in self.expressions.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            // Color indicator
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(16.0, 16.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 2.0, expr.color);

                            // Visibility toggle
                            if ui.checkbox(&mut expr.visible, "").changed() {
                                visibility_changed = true;
                            }

                            // Expression text
                            ui.label(&expr.source);

                            // Remove button
                            if ui.small_button("×").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }
                });

                if let Some(idx) = to_remove {
                    self.remove_expression(idx);
                }

                if visibility_changed {
                    self.needs_recalc = true;
                }

                ui.separator();

                // Quick add buttons
                ui.label("Quick add:");
                ui.horizontal_wrapped(|ui| {
                    let basic_examples = [
                        ("sin(x)", "y = sin(x)"),
                        ("x²", "y = x^2"),
                        ("1/x", "y = 1/x"),
                        ("circle", "x^2 + y^2 = 4"),
                    ];

                    for (label, expr) in basic_examples {
                        if ui.small_button(label).clicked() {
                            self.add_expression(expr);
                        }
                    }
                });

                ui.label("Parametric:");
                ui.horizontal_wrapped(|ui| {
                    let parametric_examples = [
                        ("unit circle", "[cos(t), sin(t)]"),
                        ("ellipse", "[2*cos(t), sin(t)]"),
                        ("lissajous", "[sin(3*t), sin(2*t)]"),
                    ];

                    for (label, expr) in parametric_examples {
                        if ui.small_button(label).clicked() {
                            self.add_expression(expr);
                        }
                    }
                });

                ui.label("Polar:");
                ui.horizontal_wrapped(|ui| {
                    let polar_examples = [
                        ("rose", "r = sin(3*theta)"),
                        ("cardioid", "r = 1 + cos(theta)"),
                        ("spiral", "r = theta/3"),
                    ];

                    for (label, expr) in polar_examples {
                        if ui.small_button(label).clicked() {
                            self.add_expression(expr);
                        }
                    }
                });

                ui.label("Inequality:");
                ui.horizontal_wrapped(|ui| {
                    let inequality_examples = [
                        ("y < x²", "y < x^2"),
                        ("y > sin(x)", "y > sin(x)"),
                        ("disk", "x^2 + y^2 < 4"),
                    ];

                    for (label, expr) in inequality_examples {
                        if ui.small_button(label).clicked() {
                            self.add_expression(expr);
                        }
                    }
                });

                ui.separator();
                ui.label("Intersections:");
                if ui.small_button("x² & x+2").clicked() {
                    self.add_expression("y = x^2");
                    self.add_expression("y = x + 2");
                }
                if ui.small_button("sin & cos").clicked() {
                    self.add_expression("y = sin(x)");
                    self.add_expression("y = cos(x)");
                }
            });

        // Settings panel (optional)
        if self.show_settings {
            egui::SidePanel::right("settings")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    ui.label("Grid");
                    ui.checkbox(
                        &mut self.canvas.grid_mut().style.show_minor_grid,
                        "Show minor grid",
                    );
                    ui.checkbox(
                        &mut self.canvas.grid_mut().style.show_labels,
                        "Show labels",
                    );

                    ui.separator();

                    ui.label("Viewport");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.label(format!("{:.2} to {:.2}",
                            self.canvas.viewport.x_min,
                            self.canvas.viewport.x_max));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        ui.label(format!("{:.2} to {:.2}",
                            self.canvas.viewport.y_min,
                            self.canvas.viewport.y_max));
                    });
                });
        }

        // Curve fitting panel
        if self.show_fitting_panel {
            let mut fit_model_changed = false;
            let mut add_to_graph = false;
            let mut clear_data = false;
            let mut toggle_fitting_mode = false;

            egui::SidePanel::right("fitting")
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.heading("Curve Fitting");
                    ui.separator();

                    // Fitting mode toggle
                    if ui.button(if self.fitting_mode { "Stop Adding Points" } else { "Add Points (Click)" }).clicked() {
                        toggle_fitting_mode = true;
                    }

                    ui.separator();

                    // Model selection
                    ui.label("Fit Model:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.fit_model == FitModel::Linear, "Linear").clicked() {
                            self.fit_model = FitModel::Linear;
                            fit_model_changed = true;
                        }
                        if ui.selectable_label(matches!(self.fit_model, FitModel::Polynomial(2)), "Quadratic").clicked() {
                            self.fit_model = FitModel::Polynomial(2);
                            fit_model_changed = true;
                        }
                        if ui.selectable_label(matches!(self.fit_model, FitModel::Polynomial(3)), "Cubic").clicked() {
                            self.fit_model = FitModel::Polynomial(3);
                            fit_model_changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.fit_model == FitModel::Exponential, "Exponential").clicked() {
                            self.fit_model = FitModel::Exponential;
                            fit_model_changed = true;
                        }
                        if ui.selectable_label(self.fit_model == FitModel::Power, "Power").clicked() {
                            self.fit_model = FitModel::Power;
                            fit_model_changed = true;
                        }
                    });

                    ui.separator();

                    // Data points
                    ui.label(format!("Data Points: {}", self.fit_data_points.len()));

                    if !self.fit_data_points.is_empty() {
                        egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                            for (i, p) in self.fit_data_points.iter().enumerate() {
                                ui.label(format!("  {}. ({:.3}, {:.3})", i + 1, p.x, p.y));
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Clear Points").clicked() {
                            clear_data = true;
                        }
                    });

                    ui.separator();

                    // Fit results
                    if let Some(ref result) = self.fit_result {
                        ui.label(egui::RichText::new("Fit Result:").strong());
                        ui.label(format!("y = {}", result.to_expression()));
                        ui.label(format!("R² = {:.6}", result.r_squared));
                        ui.label(format!("Residual Sum = {:.4}", result.residual_sum));

                        ui.separator();

                        if ui.button("Add to Graph").clicked() {
                            add_to_graph = true;
                        }
                    } else if self.fit_data_points.len() >= 2 {
                        ui.label("No valid fit (check data)");
                    } else {
                        ui.label("Need at least 2 points");
                    }
                });

            // Apply state changes after panel rendering
            if toggle_fitting_mode {
                self.fitting_mode = !self.fitting_mode;
            }
            if fit_model_changed {
                self.update_fit();
            }
            if add_to_graph {
                self.add_fit_to_expressions();
            }
            if clear_data {
                self.clear_fit_data();
            }
        }

        // Main canvas area
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(
                available_size,
                egui::Sense::click_and_drag(),
            );

            let rect = response.rect;

            // Handle interaction
            let transform = CoordinateTransform::new(
                self.canvas.viewport,
                rect.width(),
                rect.height(),
            );

            let old_viewport = self.canvas.viewport;

            // Handle fitting mode clicks before regular interaction
            if self.fitting_mode && response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Convert screen position to world coordinates
                    let local_pos = pos - rect.left_top();
                    let world_point = transform.screen_to_world(local_pos.x, local_pos.y);

                    // Add data point
                    self.fit_data_points.push(world_point);
                    self.markers.push(Marker::new(world_point, MarkerType::DataPoint));
                    self.update_fit();
                }
            } else {
                self.interaction.handle_input(&response, &mut self.canvas, &transform);
            }

            // Check if viewport changed
            if self.canvas.viewport != old_viewport {
                self.needs_recalc = true;
            }

            // Recalculate curves if needed
            if self.needs_recalc {
                self.recalculate_curves();
            }

            // Render
            self.render_canvas(&painter, rect);
        });

        // Request continuous repaints during drag
        if self.interaction.is_dragging {
            ctx.request_repaint();
        }
    }
}
