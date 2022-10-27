use crate::RenderMode;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub width: usize,
    pub height: usize,
    #[arg(value_enum)]
    pub mode: RenderMode,
    #[arg(default_value_t = 1)]
    pub samples: usize,
}
