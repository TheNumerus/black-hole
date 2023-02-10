use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to file with specified tests
    pub config_path: PathBuf,
}
