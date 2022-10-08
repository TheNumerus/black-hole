use crate::RenderMode;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub width: usize,
    pub height: usize,
    #[arg(value_enum)]
    pub mode: RenderMode,
    #[arg(long)]
    pub multisampled: bool,
}
