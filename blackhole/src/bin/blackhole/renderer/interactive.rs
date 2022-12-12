use blackhole::filter::{BlackmanHarrisFilter, PixelFilter};
use blackhole::frame::{Frame, Region};
use blackhole::framebuffer::{FrameBuffer, Pixel};
use blackhole::marcher::RayMarcher;
use blackhole::scene::Scene;
use blackhole::RenderMode;
use rayon::prelude::*;

use std::sync::atomic::Ordering;

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

use crate::renderer::{Scaling, MAX_STEPS_PER_SAMPLE, TOTAL_STEPS};

pub struct InteractiveRenderer {
    pub ray_marcher: RayMarcher,
    pub samples: usize,
    pub threads: usize,
    pub frame: Frame,
    pub filter: Box<dyn PixelFilter>,
    pub scaling: Scaling,
}

impl InteractiveRenderer {
    pub fn render(
        &mut self,
        front_fb: Arc<RwLock<FrameBuffer>>,
        tx: Sender<RenderOutMsg>,
        rx: Receiver<RenderInMsg>,
    ) -> () {
        let mut should_render = false;

        let mut back_fb = FrameBuffer::new(self.frame.width, self.frame.height);

        let mut scene: Option<Scene> = None;

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build()
            .expect("Failed to build rendering threadpool");

        'main: loop {
            if should_render && scene.is_some() {
                let mut scene_unwrapped = scene.as_mut().unwrap();
                let max_step = scene_unwrapped.max_possible_step(scene_unwrapped.camera.location);
                for i in 0..self.samples {
                    if let Some(msg) = rx.try_iter().next() {
                        match msg {
                            RenderInMsg::Resize(w, h) => {
                                let (w, h) = (w / self.scaling.scale(), h / self.scaling.scale());

                                self.frame.width = w as usize;
                                self.frame.height = h as usize;
                                back_fb = FrameBuffer::new(self.frame.width, self.frame.height);
                                {
                                    let mut write_lock = front_fb.write().unwrap();

                                    *write_lock =
                                        FrameBuffer::new(self.frame.width, self.frame.height);
                                }
                                continue 'main;
                            }
                            RenderInMsg::SceneChange(s) => {
                                scene = Some(s);
                                should_render = true;
                                continue 'main;
                            }
                            RenderInMsg::Restart => {
                                should_render = true;
                                continue 'main;
                            }
                            RenderInMsg::Exit => {
                                break 'main;
                            }
                        }
                    }

                    let offset = self.filter.next().unwrap();

                    {
                        let read_lock = front_fb.read().unwrap();

                        if self.threads == 1 {
                            for (y, (slice_out, slice_in)) in back_fb
                                .buffer_mut()
                                .chunks_mut(self.frame.width)
                                .zip(read_lock.buffer().chunks(self.frame.width))
                                .enumerate()
                            {
                                self.scanline(
                                    scene_unwrapped,
                                    max_step,
                                    y,
                                    slice_in,
                                    slice_out,
                                    0,
                                    i,
                                    offset,
                                );
                            }
                        } else {
                            pool.install(|| {
                                back_fb
                                    .buffer_mut()
                                    .par_chunks_mut(self.frame.width)
                                    .zip(read_lock.buffer().par_chunks(self.frame.width))
                                    .enumerate()
                                    .for_each(|(y, (slice_out, slice_in))| {
                                        self.scanline(
                                            scene_unwrapped,
                                            max_step,
                                            y,
                                            slice_in,
                                            slice_out,
                                            0,
                                            i,
                                            offset,
                                        )
                                    })
                            });
                        }
                    }

                    {
                        let mut write_lock = front_fb.write().unwrap();

                        std::mem::swap(&mut back_fb, &mut write_lock);
                    }
                    tx.send(RenderOutMsg::Update).unwrap();
                }
            }

            let msg = rx.recv().unwrap();

            match msg {
                RenderInMsg::Resize(w, h) => {
                    let (w, h) = (w / self.scaling.scale(), h / self.scaling.scale());

                    self.frame.width = w as usize;
                    self.frame.height = h as usize;
                    back_fb = FrameBuffer::new(self.frame.width, self.frame.height);
                    {
                        let mut write_lock = front_fb.write().unwrap();

                        *write_lock = FrameBuffer::new(self.frame.width, self.frame.height);
                    }
                    continue 'main;
                }
                RenderInMsg::SceneChange(s) => {
                    scene = Some(s);
                    should_render = true;
                    continue 'main;
                }
                RenderInMsg::Restart => {
                    should_render = true;
                    continue 'main;
                }
                RenderInMsg::Exit => {
                    break 'main;
                }
            }
        }
    }

    fn scanline(
        &self,
        scene: &Scene,
        max_step: f64,
        y: usize,
        slice_input: &[Pixel],
        slice_output: &mut [Pixel],
        slice_start: usize,
        sample: usize,
        offset: (f64, f64),
    ) {
        for (x, pixel) in slice_input.iter().enumerate() {
            if let Region::Window {
                x_min,
                x_max,
                y_min,
                y_max,
            } = self.frame.region
            {
                if x >= x_max || x < x_min || y >= y_max || y < y_min {
                    continue;
                }
            }

            let rel_x = ((x + slice_start) as f64 + offset.0) / (self.frame.width as f64);
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
                slice_output[x] += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
            } else {
                let base = *pixel;

                let color = Pixel::from(sample_info.color);

                slice_output[x] = base * (sample as f32 / (sample as f32 + 1.0))
                    + color * (1.0 / (sample as f32 + 1.0));
            }
        }
    }
}

impl Default for InteractiveRenderer {
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

pub enum RenderInMsg {
    Resize(u32, u32),
    SceneChange(Scene),
    Restart,
    Exit,
}

pub enum RenderOutMsg {
    Update,
}
