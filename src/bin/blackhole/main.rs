use std::fs::File;
use std::io::BufWriter;

use cgmath::{InnerSpace, Vector3};

use clap::Parser;

use blackhole::camera::Camera;
use blackhole::framebuffer::{FrameBuffer, Pixel};
use blackhole::PixelFilter;

mod args;
mod renderer;
mod scene_loader;
mod shaders;

use crate::renderer::{Region, Renderer};
use args::Args;
use renderer::RenderMode;
use scene_loader::SceneLoader;

fn main() {
    // clion needs help in trait annotation
    let args = <Args as Parser>::parse();

    let mut fb = FrameBuffer::new(args.width, args.height);

    let loader = SceneLoader::new();

    let scene = loader.load_path(args.scene);

    let scene = match scene {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could not read scene description: {e}");
            std::process::exit(-1);
        }
    };

    let camera = setup_camera(args.width as f64, args.height as f64);

    /*let region = Region::Window {
        x_min: 499,
        y_min: 700,
        x_max: 500,
        y_max: 701,
    };
    let region = Region::Window {
        x_min: 450,
        y_min: 650,
        x_max: 550,
        y_max: 750,
    };*/
    let region = Region::Whole;

    let mut renderer = Renderer {
        mode: args.mode,
        samples: args.samples,
        threads: args.threads,
        max_steps: 2 << 16,
        max_depth: 16,
        width: args.width,
        height: args.height,
        sampler: PixelFilter::new(1.5),
        region,
    };

    renderer.render(&scene, &camera, &mut fb);

    post_process(&mut fb, &args.mode);

    write_out(fb, args.width as u32, args.height as u32);
}

fn post_process(fb: &mut FrameBuffer, mode: &RenderMode) {
    let luminance_base = Vector3::new(0.2126, 0.7152, 0.0722);

    match mode {
        RenderMode::Shaded => {
            for pixel in fb.buffer_mut() {
                let luminance = Vector3::new(pixel.r, pixel.g, pixel.b).dot(luminance_base);

                let new_luminance = luminance / (luminance + 1.0);

                let tonemapped = Pixel::new(
                    pixel.r * (new_luminance / luminance),
                    pixel.g * (new_luminance / luminance),
                    pixel.b * (new_luminance / luminance),
                    pixel.a,
                );

                let new_pixel = Pixel::new(
                    tonemapped.r.powf(1.0 / 2.2),
                    tonemapped.g.powf(1.0 / 2.2),
                    tonemapped.b.powf(1.0 / 2.2),
                    pixel.a,
                );

                *pixel = new_pixel;
            }
        }
        RenderMode::Samples | RenderMode::Normal => {}
    }
}

fn write_out(fb: FrameBuffer, width: u32, height: u32) {
    let buf = unsafe {
        assert_eq!(std::mem::size_of::<Pixel>(), 4 * std::mem::size_of::<f32>());

        fb.as_f32_slice()
    };

    let mapped = buf.iter().map(|e| (e * 255.0) as u8).collect::<Vec<_>>();

    let file = File::create("out.png").unwrap();
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&mapped).unwrap();
}

fn setup_camera(width: f64, height: f64) -> Camera {
    let mut camera = Camera::new();
    camera.location = Vector3::new(0.0, 0.54, 10.0);
    camera.hor_fov = 42.0;
    camera.up(Vector3::new(0.1, 1.0, 0.0));
    camera.set_forward(Vector3::new(0.0, -0.01, -1.0));
    camera.aspect_ratio = width / height;

    //let mut camera = Camera::new();
    /*camera.location = Vector3::new(0.0, 10.0, 0.0);
    camera.hor_fov = 42.0;
    camera.up(Vector3::new(1.0, 0.0, 0.0));
    camera.set_forward(Vector3::new(0.0, -1.01, 0.0));
    camera.aspect_ratio = width / height;*/
    camera
}
