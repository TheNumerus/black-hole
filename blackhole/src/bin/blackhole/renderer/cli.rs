use blackhole::filter::{BlackmanHarrisFilter, PixelFilter};
use blackhole::frame::{Frame, Region};
use blackhole::framebuffer::{FrameBuffer, Pixel};
use blackhole::marcher::RayMarcher;
use blackhole::scene::Scene;
use blackhole::RenderMode;

use std::io::Write;
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

            if self.threads == 1 {
                for (y, slice) in fb.buffer_mut().chunks_mut(self.frame.width).enumerate() {
                    self.scanline(scene, max_step, y, slice, i, offset);
                }
            } else {
                pool.install(|| {
                    fb.buffer_mut()
                        .par_chunks_mut(self.frame.width)
                        .enumerate()
                        .for_each(|(y, slice)| self.scanline(scene, max_step, y, slice, i, offset))
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

                    let value = sample_count / 256.0 as f32 / self.samples as f32;

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

    fn scanline(
        &self,
        scene: &Scene,
        max_step: f64,
        y: usize,
        slice: &mut [Pixel],
        sample: usize,
        offset: (f64, f64),
    ) {
        if let Region::Window { y_min, y_max, .. } = self.frame.region {
            if y >= y_max || y < y_min {
                return;
            }
        }

        for (x, pixel) in slice.iter_mut().enumerate() {
            if let Region::Window { x_min, x_max, .. } = self.frame.region {
                if x >= x_max || x < x_min {
                    continue;
                }
            }

            let rel_x = (x as f64 + offset.0) / (self.frame.width as f64);
            let rel_y = (y as f64 + offset.1) / (self.frame.height as f64);

            let sample_info = self.ray_marcher.color_for_ray(
                scene
                    .camera
                    .cast_ray(rel_x, rel_y, self.frame.aspect_ratio()),
                &scene,
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
