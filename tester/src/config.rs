use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub test: Vec<Test>,
}

#[derive(Deserialize, Debug)]
pub struct Test {
    #[serde(rename = "scene")]
    pub scene_path: PathBuf,
    #[serde(rename = "original")]
    pub original_image: PathBuf,
    pub width: usize,
    pub height: usize,
    pub samples: usize,
}
