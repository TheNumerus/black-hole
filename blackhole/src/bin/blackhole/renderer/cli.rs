use blackhole::filter::{BlackmanHarrisFilter, PixelFilter};
use blackhole::frame::{Frame, Region};
use blackhole::framebuffer::{FrameBuffer, Pixel};
use blackhole::marcher::RayMarcher;
use blackhole::scene::Scene;
use blackhole::RenderMode;

use std::io::Write;
use std::slice::ChunksMut;
use std::sync::atomic::Ordering;

use rayon::prelude::*;

use crate::renderer::{Scaling, MAX_STEPS_PER_SAMPLE, TOTAL_STEPS};

pub struct CliRenderer {
    pub ray_marcher: RayMarcher,
    pub samples: usize,
    pub threads: usize,
    pub frame: Frame,
    pub filter: Box<dyn PixelFilter>,
    pub scaling: Scaling,
}

impl CliRenderer {
    pub fn render(&mut self, scene: &Scene, fb: &mut FrameBuffer) {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build()
            .expect("Failed to build rendering threadpool");

        let start = std::time::Instant::now();

        let max_step = scene.max_possible_step(scene.camera.location);

        let mut max_step_count = 0;

        TOTAL_STEPS.store(0, Ordering::SeqCst);

        for i in 0..self.samples {
            let offset = self.filter.next().unwrap();
            let fbi = FrameBufferIterator::from_framebuffer(fb, self.frame.region);

            if self.threads == 1 {
                for slice in fbi {
                    self.scanline(scene, max_step, slice, i, offset);
                }
            } else {
                pool.install(|| {
                    fbi.par_bridge()
                        .for_each(|slice| self.scanline(scene, max_step, slice, i, offset));
                });
            }

            max_step_count += MAX_STEPS_PER_SAMPLE.load(Ordering::SeqCst);

            let sample_end = std::time::Instant::now();
            let remaining_part = self.samples as f32 / (i as f32 + 1.0) - 1.0;
            let time = sample_end - start;
            let remaining_time = time.mul_f32(remaining_part);
            print!(
                "\rSample {}/{}, time: {:02}:{:02}, remaining: {:02}:{:02}",
                i + 1,
                self.samples,
                time.as_secs() / 60,
                time.as_secs() % 60,
                remaining_time.as_secs() / 60,
                remaining_time.as_secs() % 60
            );
            std::io::stdout().flush().expect("Failed to flush stdout");
        }

        println!();

        if let RenderMode::Samples = self.ray_marcher.mode {
            for y in 0..self.frame.height {
                for x in 0..self.frame.width {
                    let pixel = fb.pixel_mut(x, y).unwrap();

                    let sample_count = pixel.r;

                    let value = sample_count / 256.0 / self.samples as f32;

                    *pixel = Pixel::new(value, 1.0 - value, 0.0, 1.0);
                }
            }
        }

        let end = std::time::Instant::now();

        println!("Render took {:.02} seconds", (end - start).as_secs_f64());
        println!("Max steps: {max_step_count}");
        println!(
            "Avg steps per pixel: {}",
            TOTAL_STEPS.load(Ordering::SeqCst) as f64
                / (self.frame.width * self.frame.height) as f64
        );
    }

    fn scanline<'fb>(
        &self,
        scene: &Scene,
        max_step: f64,
        slice: FrameBufferSlice<'fb>,
        sample: usize,
        offset: (f64, f64),
    ) {
        let rel_y = (slice.y as f64 + offset.1) / (self.frame.height as f64);
        for (x, pixel) in slice.slice.iter_mut().enumerate() {
            let rel_x = ((x + slice.x_start) as f64 + offset.0) / (self.frame.width as f64);

            let sample_info = self.ray_marcher.color_for_ray(
                scene
                    .camera
                    .cast_ray(rel_x, rel_y, self.frame.aspect_ratio()),
                scene,
                max_step,
                0,
            );

            MAX_STEPS_PER_SAMPLE.fetch_max(sample_info.steps, Ordering::SeqCst);
            TOTAL_STEPS.fetch_add(sample_info.steps, Ordering::SeqCst);
            if let RenderMode::Samples = self.ray_marcher.mode {
                *pixel += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
            } else {
                let base = *pixel;

                let color = Pixel::from(sample_info.color);

                *pixel = base * (sample as f32 / (sample as f32 + 1.0))
                    + color * (1.0 / (sample as f32 + 1.0));
            }
        }
    }
}

impl Default for CliRenderer {
    fn default() -> Self {
        Self {
            ray_marcher: RayMarcher::default(),
            samples: 128,
            threads: 0,
            frame: Frame {
                width: 1280,
                height: 720,
                region: Region::Whole,
            },
            filter: Box::new(BlackmanHarrisFilter::new(1.5)),
            scaling: Default::default(),
        }
    }
}

struct FrameBufferSlice<'fb> {
    slice: &'fb mut [Pixel],
    y: usize,
    x_start: usize,
}

struct FrameBufferIterator<'fb> {
    chunks: ChunksMut<'fb, Pixel>,
    start: usize,
    end: usize,
    line: usize,
}

impl<'fb> FrameBufferIterator<'fb> {
    pub fn from_framebuffer(fb: &'fb mut FrameBuffer, region: Region) -> Self {
        let width = fb.width();
        match region {
            Region::Whole => Self {
                start: 0,
                end: fb.width(),
                line: 0,
                chunks: fb.buffer_mut().chunks_mut(width),
            },
            Region::Window {
                x_min,
                x_max,
                y_min,
                y_max,
            } => Self {
                start: x_min,
                end: x_max - x_min,
                line: y_min,
                chunks: fb.buffer_mut()[y_min * width..y_max * width].chunks_mut(width),
            },
        }
    }
}

impl<'fb> Iterator for FrameBufferIterator<'fb> {
    type Item = FrameBufferSlice<'fb>;

    fn next(&mut self) -> Option<FrameBufferSlice<'fb>> {
        if let Some(slice) = self.chunks.next() {
            let slice = &mut slice[self.start..(self.start + self.end)];

            self.line += 1;

            Some(FrameBufferSlice {
                slice,
                y: self.line - 1,
                x_start: self.start,
            })
        } else {
            None
        }
    }
}
