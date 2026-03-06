//! Parameter sliders for interactive expressions

/// Slider configuration
#[derive(Debug, Clone)]
pub struct SliderConfig {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Step size (0 for continuous)
    pub step: f64,
    /// Default value
    pub default: f64,
    /// Show value label
    pub show_value: bool,
    /// Number of decimal places to display
    pub precision: usize,
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            min: -10.0,
            max: 10.0,
            step: 0.0,
            default: 1.0,  // Default to 1 for mathematical expressions
            show_value: true,
            precision: 2,
        }
    }
}

impl SliderConfig {
    pub fn new(min: f64, max: f64) -> Self {
        // Default to 1.0 if within range, otherwise use midpoint
        let default = if min <= 1.0 && 1.0 <= max {
            1.0
        } else {
            (min + max) / 2.0
        };
        Self {
            min,
            max,
            default,
            ..Default::default()
        }
    }

    pub fn with_default(mut self, default: f64) -> Self {
        self.default = default.clamp(self.min, self.max);
        self
    }

    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn with_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }
}

/// A parameter slider
#[derive(Debug, Clone)]
pub struct ParameterSlider {
    /// Parameter name
    pub name: String,
    /// Current value
    pub value: f64,
    /// Configuration
    pub config: SliderConfig,
    /// Is the slider currently being dragged
    pub dragging: bool,
    /// Is this slider animating
    pub animating: bool,
    /// Animation speed (units per second)
    pub animation_speed: f64,
    /// Animation direction (1 or -1)
    pub animation_direction: i8,
}

impl ParameterSlider {
    pub fn new(name: impl Into<String>, config: SliderConfig) -> Self {
        let value = config.default;
        Self {
            name: name.into(),
            value,
            config,
            dragging: false,
            animating: false,
            animation_speed: 1.0,
            animation_direction: 1,
        }
    }

    /// Create with default configuration
    pub fn with_range(name: impl Into<String>, min: f64, max: f64) -> Self {
        Self::new(name, SliderConfig::new(min, max))
    }

    /// Set the value, clamping to range
    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(self.config.min, self.config.max);

        // Snap to step if configured
        if self.config.step > 0.0 {
            let steps = ((self.value - self.config.min) / self.config.step).round();
            self.value = self.config.min + steps * self.config.step;
        }
    }

    /// Reset to default value
    pub fn reset(&mut self) {
        self.value = self.config.default;
        self.animating = false;
    }

    /// Start animation
    pub fn start_animation(&mut self) {
        self.animating = true;
    }

    /// Stop animation
    pub fn stop_animation(&mut self) {
        self.animating = false;
    }

    /// Toggle animation
    pub fn toggle_animation(&mut self) {
        self.animating = !self.animating;
    }

    /// Update animation (call each frame)
    pub fn update(&mut self, dt: f64) {
        if !self.animating {
            return;
        }

        let delta = self.animation_speed * self.animation_direction as f64 * dt;
        let new_value = self.value + delta;

        // Bounce at edges
        if new_value >= self.config.max {
            self.value = self.config.max;
            self.animation_direction = -1;
        } else if new_value <= self.config.min {
            self.value = self.config.min;
            self.animation_direction = 1;
        } else {
            self.value = new_value;
        }
    }

    /// Get the normalized position (0 to 1)
    pub fn normalized(&self) -> f64 {
        (self.value - self.config.min) / (self.config.max - self.config.min)
    }

    /// Set from normalized position (0 to 1)
    pub fn set_normalized(&mut self, t: f64) {
        let value = self.config.min + t * (self.config.max - self.config.min);
        self.set_value(value);
    }

    /// Format the current value for display
    pub fn format_value(&self) -> String {
        format!("{:.prec$}", self.value, prec = self.config.precision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_creation() {
        let slider = ParameterSlider::with_range("a", -5.0, 5.0);
        assert_eq!(slider.name, "a");
        assert_eq!(slider.value, 1.0);  // Default is now 1.0
        assert_eq!(slider.config.min, -5.0);
        assert_eq!(slider.config.max, 5.0);
    }

    #[test]
    fn test_slider_clamping() {
        let mut slider = ParameterSlider::with_range("a", 0.0, 10.0);
        slider.set_value(15.0);
        assert_eq!(slider.value, 10.0);

        slider.set_value(-5.0);
        assert_eq!(slider.value, 0.0);
    }

    #[test]
    fn test_slider_stepping() {
        let config = SliderConfig::new(0.0, 10.0).with_step(0.5);
        let mut slider = ParameterSlider::new("a", config);

        slider.set_value(3.3);
        assert!((slider.value - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_slider_normalized() {
        let slider = ParameterSlider::with_range("a", 0.0, 100.0);
        // Default is 1.0, normalized = (1 - 0) / (100 - 0) = 0.01
        assert!((slider.normalized() - 0.01).abs() < 0.001);

        let mut slider2 = ParameterSlider::with_range("b", -10.0, 10.0);
        slider2.set_normalized(0.75);
        assert!((slider2.value - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_slider_animation() {
        let mut slider = ParameterSlider::with_range("a", 0.0, 10.0);
        slider.set_value(5.0);
        slider.animation_speed = 2.0;
        slider.start_animation();

        slider.update(1.0);
        assert!((slider.value - 7.0).abs() < 0.001);

        // Test bouncing
        slider.set_value(9.5);
        slider.animation_direction = 1;
        slider.update(1.0);
        assert_eq!(slider.animation_direction, -1);
    }
}
