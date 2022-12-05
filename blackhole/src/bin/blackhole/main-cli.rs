use std::fs::File;
use std::io::BufWriter;

use cgmath::{InnerSpace, Vector3};

use clap::Parser;

use blackhole::filter::BlackmanHarrisFilter;
use blackhole::framebuffer::{FrameBuffer, Pixel};

mod args;
mod renderer;
mod scene_loader;
mod shaders;

use args::Args;
use blackhole::frame::{Frame, Region};
use renderer::{RenderMode, Renderer};
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

    let mut renderer = Renderer {
        mode: args.mode,
        samples: args.samples,
        threads: args.threads,
        filter: Box::new(BlackmanHarrisFilter::new(1.5)),
        frame: Frame {
            width: args.width,
            height: args.height,
            region: Region::Whole,
        },
        ..Default::default()
    };

    renderer.render(&scene, &mut fb);

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
