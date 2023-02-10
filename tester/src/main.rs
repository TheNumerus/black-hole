use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;

mod args;
mod config;

use crate::config::Test;

fn main() {
    let args = <args::Args as Parser>::parse();
    let config = std::fs::read_to_string(&args.config_path).expect("Cannot read config file");

    let tests: config::Config = toml::from_str(&config).expect("Invalid config structure");
    let tests = tests.test;

    let test_path = get_test_path(&args.config_path);

    for test in &tests {
        println!("Testing {:?}", test.scene_path);

        let test_res = execute_test(&test_path, test).unwrap();
        let new_img = read_image(&test_path, &test_res);
        let old_img = read_image(&test_path, &test.original_image);

        let comp = compare(&new_img, &old_img);

        println!(
            "Total error: {}\nPercentage error: {}%",
            comp.total_err, comp.percentage_err
        );
    }
}

fn get_test_path(config_path: impl AsRef<Path>) -> PathBuf {
    config_path
        .as_ref()
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn execute_test(wd: impl AsRef<Path>, test: &Test) -> Result<PathBuf, ()> {
    let mut file_name = test.original_image.file_stem().unwrap().to_owned();
    file_name.push("_output.");
    file_name.push(test.original_image.extension().unwrap());

    let output_name = test.original_image.with_file_name(file_name);

    let mut cmd = Command::new("../target/release/blackhole-cli")
        .current_dir(wd)
        .arg(&test.scene_path)
        .args([
            "--width",
            &test.width.to_string(),
            "--height",
            &test.height.to_string(),
            "--samples",
            &test.samples.to_string(),
            "--output",
        ])
        .arg(&output_name)
        .spawn()
        .unwrap();

    cmd.wait().unwrap();

    Ok(output_name)
}

fn read_image(test_path: impl AsRef<Path>, path: impl AsRef<Path>) -> Vec<u8> {
    let mut img_path = test_path.as_ref().to_owned();
    img_path.push(path.as_ref());
    let file = File::open(img_path).unwrap();

    let decoder = png::Decoder::new(file);

    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    buf
}

fn compare(new_img: &[u8], old_img: &[u8]) -> Comparison {
    if new_img.len() != old_img.len() {
        panic!("sizes do not match");
    }

    let mut total_err = 0.0;

    for (n, o) in new_img.iter().zip(old_img.iter()) {
        total_err += n.abs_diff(*o) as f32 / 255.0;
    }

    let percentage_err = (total_err / new_img.len() as f32) * 100.0;

    Comparison {
        total_err,
        percentage_err,
    }
}

struct Comparison {
    pub total_err: f32,
    pub percentage_err: f32,
}
