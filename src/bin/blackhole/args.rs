use crate::RenderMode;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg()]
    pub scene: PathBuf,
    #[arg(short, long, default_value_t = 1280)]
    pub width: usize,
    #[arg(short, long, default_value_t = 720)]
    pub height: usize,
    #[arg(value_enum, default_value_t = RenderMode::Shaded)]
    pub mode: RenderMode,
    #[arg(short, long, default_value_t = 1)]
    pub samples: usize,
    #[arg(short, long, default_value_t = 0)]
    pub threads: usize,
}