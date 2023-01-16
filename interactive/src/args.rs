use clap::{Parser, ValueEnum};

use crate::renderer::Scaling;
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
    #[arg(value_enum, short = 'X', default_value_t = ScalingArg::X1)]
    pub scaling: ScalingArg,
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

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ScalingArg {
    #[value(name = "1")]
    X1 = 1,
    #[value(name = "2")]
    X2 = 2,
    #[value(name = "4")]
    X4 = 4,
    #[value(name = "8")]
    X8 = 8,
}

impl From<ScalingArg> for Scaling {
    fn from(r: ScalingArg) -> Self {
        match r {
            ScalingArg::X1 => Self::X1,
            ScalingArg::X2 => Self::X2,
            ScalingArg::X4 => Self::X4,
            ScalingArg::X8 => Self::X8,
        }
    }
}
