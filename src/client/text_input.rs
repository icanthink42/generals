#[cfg(target_arch = "wasm32")]
use {
    std::rc::Rc,
    web_sys::{CanvasRenderingContext2d, TextMetrics},
    wasm_bindgen::{JsCast, prelude::*},
};

#[cfg(target_arch = "wasm32")]
pub struct TextInput {
    pub text: String,
    pub placeholder: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub enabled: bool,
    pub focused: bool,
    pub on_change: Rc<dyn Fn(&str)>,
}

#[cfg(target_arch = "wasm32")]
impl TextInput {
    pub fn new(
        placeholder: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        on_change: Rc<dyn Fn(&str)>,
    ) -> Self {
        Self {
            text: String::new(),
            placeholder,
            x,
            y,
            width,
            height,
            enabled: true,
            focused: false,
            on_change,
        }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    pub fn handle_click(&mut self, x: f64, y: f64) -> bool {
        if self.enabled && self.contains(x, y) {
            self.focused = true;
            true
        } else {
            self.focused = false;
            false
        }
    }

    pub fn handle_key(&mut self, key: &str) {
        if !self.focused || !self.enabled {
            return;
        }

        match key {
            "Backspace" => {
                if !self.text.is_empty() {
                    self.text.pop();
                    (self.on_change)(&self.text);
                }
            }
            key if key.len() == 1 => {
                self.text.push_str(key);
                (self.on_change)(&self.text);
            }
            _ => {}
        }
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) {
        // Draw input background
        context.set_fill_style_str(if self.enabled {
            if self.focused { "#404040" } else { "#303030" }
        } else {
            "#202020"
        });
        context.fill_rect(self.x, self.y, self.width, self.height);

        // Draw border
        context.set_stroke_style_str(if self.focused { "#4CAF50" } else { "#404040" });
        context.set_line_width(2.0);
        context.stroke_rect(self.x, self.y, self.width, self.height);

        // Draw text
        context.set_font("16px Arial");
        context.set_fill_style_str("white");
        context.set_text_align("left");
        context.set_text_baseline("middle");

        let text = if self.text.is_empty() {
            context.set_fill_style_str("#808080"); // Gray for placeholder
            &self.placeholder
        } else {
            &self.text
        };

        let _ = context.fill_text(
            text,
            self.x + 10.0,  // Add padding
            self.y + self.height / 2.0,
        );

        // Draw cursor when focused
        if self.focused {
                            let text_width = if self.text.is_empty() {
                    0.0
                } else {
                    let metrics = context.measure_text(&self.text).unwrap();
                    metrics.width()
                };

            context.set_fill_style_str("white");
            context.fill_rect(
                self.x + 10.0 + text_width,  // After text
                self.y + 5.0,                // Small padding from top
                2.0,                         // Cursor width
                self.height - 10.0,          // Height minus padding
            );
        }
    }
}
