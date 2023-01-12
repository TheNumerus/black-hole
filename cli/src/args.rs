use clap::{Parser, ValueEnum};

use blackhole::RenderMode;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to scene JSON file
    #[arg()]
    pub scene: PathBuf,
    /// Width of the output image
    #[arg(long, default_value_t = 1280)]
    pub width: usize,
    /// Height of the output image
    #[arg(long, default_value_t = 720)]
    pub height: usize,
    /// Render setting, used for debugging
    #[arg(value_enum, default_value_t = RenderModeArg::Shaded)]
    pub mode: RenderModeArg,
    /// Amount of samples to render
    #[arg(short, long, default_value_t = 128)]
    pub samples: usize,
    /// Threads to use for rendering (0 for automatic setting)
    #[arg(short, long, default_value_t = 0)]
    pub threads: usize,
    /// Path to save render to
    #[arg(short, long, default_value_os_t = PathBuf::from("out.png"))]
    pub output: PathBuf,
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
