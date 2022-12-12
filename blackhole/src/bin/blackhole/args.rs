use clap::{Parser, ValueEnum};

use blackhole::RenderMode;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg()]
    pub scene: PathBuf,
    #[arg(long, default_value_t = 1280)]
    pub width: usize,
    #[arg(long, default_value_t = 720)]
    pub height: usize,
    #[arg(value_enum, default_value_t = RenderModeArg::Shaded)]
    pub mode: RenderModeArg,
    #[arg(short, long, default_value_t = 1)]
    pub samples: usize,
    #[arg(short, long, default_value_t = 0)]
    pub threads: usize,
}

#[derive(Debug, Parser)]
pub struct ArgsInteractive {
    #[arg(value_enum, default_value_t = RenderModeArg::Shaded)]
    pub mode: RenderModeArg,
    #[arg(short, long, default_value_t = 1)]
    pub samples: usize,
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
