use std::sync::atomic::AtomicUsize;

static TOTAL_STEPS: AtomicUsize = AtomicUsize::new(0);
static MAX_STEPS_PER_SAMPLE: AtomicUsize = AtomicUsize::new(0);

mod cli;
mod interactive;

pub use cli::CliRenderer;
pub use interactive::{InteractiveRenderer, RenderInMsg, RenderOutMsg};

pub enum Scaling {
    X1,
    X2,
    X4,
    X8,
}

impl Scaling {
    pub const fn scale(&self) -> u32 {
        match self {
            Scaling::X1 => 1,
            Scaling::X2 => 2,
            Scaling::X4 => 4,
            Scaling::X8 => 8,
        }
    }
}

impl Default for Scaling {
    fn default() -> Self {
        Self::X1
    }
}
