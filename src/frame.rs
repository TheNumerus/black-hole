pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub region: Region,
}

impl Frame {
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

#[derive(Copy, Clone)]
pub enum Region {
    Whole,
    Window {
        x_min: usize,
        y_min: usize,
        x_max: usize,
        y_max: usize,
    },
}
