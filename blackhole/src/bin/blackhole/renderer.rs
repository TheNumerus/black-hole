use std::sync::atomic::AtomicUsize;

static TOTAL_STEPS: AtomicUsize = AtomicUsize::new(0);
static MAX_STEPS_PER_SAMPLE: AtomicUsize = AtomicUsize::new(0);

mod cli;
mod interactive;

pub use cli::CliRenderer;
pub use interactive::{InteractiveRenderer, RenderInMsg, RenderOutMsg};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Scaling {
    X1,
    X2,
    X4,
    X8,
}

impl Scaling {
    pub const fn scale(&self) -> u32 {
        match self {
            Self::X1 => 1,
            Self::X2 => 2,
            Self::X4 => 4,
            Self::X8 => 8,
        }
    }

    pub const fn lower(&self) -> Self {
        match self {
            Self::X1 => Self::X1,
            Self::X2 => Self::X1,
            Self::X4 => Self::X2,
            Self::X8 => Self::X4,
        }
    }
}

impl Default for Scaling {
    fn default() -> Self {
        Self::X1
    }
}
