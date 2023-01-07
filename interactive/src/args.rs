use clap::{Parser, ValueEnum};

use blackhole::RenderMode;

#[derive(Debug, Parser)]
pub struct ArgsInteractive {
    /// Render setting, used for debugging
    #[arg(value_enum, default_value_t = RenderModeArg::Shaded)]
    pub mode: RenderModeArg,
    /// Amount of samples to render
    #[arg(short, long, default_value_t = 128)]
    pub samples: usize,
    /// Threads to use for rendering (0 for automatic setting)
    #[arg(short, long, default_value_t = 0)]
    pub threads: usize,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum RenderModeArg {
    Samples,
    Normal,
    Shaded,
}

impl From<RenderModeArg> for RenderMode {
    fn from(r: RenderModeArg) -> Self {
        match r {
            RenderModeArg::Samples => Self::Samples,
            RenderModeArg::Normal => Self::Normal,
            RenderModeArg::Shaded => Self::Shaded,
        }
    }
}
