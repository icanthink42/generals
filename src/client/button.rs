#[cfg(target_arch = "wasm32")]
use {
    std::rc::Rc,
    web_sys::CanvasRenderingContext2d,
};

#[cfg(target_arch = "wasm32")]
pub struct Button {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub enabled: bool,
    pub callback: Rc<dyn Fn()>,
}

#[cfg(target_arch = "wasm32")]
impl Button {
    pub fn new(text: String, x: f64, y: f64, width: f64, height: f64, callback: Rc<dyn Fn()>) -> Self {
        Self {
            text,
            x,
            y,
            width,
            height,
            enabled: true,
            callback,
        }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) {
        // Draw button background
        context.set_fill_style_str(if self.enabled { "#4CAF50" } else { "#808080" });  // Green when enabled, gray when disabled
        context.fill_rect(self.x, self.y, self.width, self.height);

        // Draw button text
        context.set_font("20px Arial");
        context.set_fill_style_str("white");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        let _ = context.fill_text(
            &self.text,
            self.x + self.width / 2.0,
            self.y + self.height / 2.0,
        );
    }
}
