use blackhole::filter::{BlackmanHarrisFilter, PixelFilter};
use blackhole::frame::{Frame, Region};
use blackhole::framebuffer::{FrameBuffer, Pixel};
use blackhole::marcher::RayMarcher;
use blackhole::scene::Scene;
use blackhole::RenderMode;

use flume::{Receiver, RecvError, Sender};

use rayon::prelude::*;

use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::renderer::Scaling;

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
    ) {
        let mut back_fb = FrameBuffer::new(self.frame.width, self.frame.height);

        let mut scene: Option<Scene> = None;

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build()
            .expect("Failed to build rendering threadpool");

        let mut current_scale;
        let mut window_size = (self.frame.width, self.frame.height);

        let mut last_update = Instant::now();

        'jobs: loop {
            let msg = rx.recv();

            match Self::msg_to_actions(msg) {
                RendererActions::Exit => break 'jobs,
                RendererActions::Restart {
                    resize_buffers,
                    scene_change,
                } => {
                    if let Some((w, h)) = resize_buffers {
                        window_size = (w as usize, h as usize);
                        back_fb = FrameBuffer::new(w as usize, h as usize);
                        {
                            let mut write_lock = front_fb.write().unwrap();

                            *write_lock = FrameBuffer::new(w as usize, h as usize);
                        }
                    }

                    if let Some(scene_new) = scene_change {
                        scene = Some(scene_new);
                    }

                    current_scale = Scaling::X8;
                    let (w, h) = (
                        window_size.0 as u32 / current_scale.scale(),
                        window_size.1 as u32 / current_scale.scale(),
                    );

                    self.frame.width = w as usize;
                    self.frame.height = h as usize;
                }
            }
            if let Some(scene) = &scene {
                let max_step = scene.max_possible_step(scene.camera.location);

                let mut sample = 0;
                self.filter.reset();

                'sample: loop {
                    if sample >= self.samples || !rx.is_empty() {
                        break 'sample;
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
                                .take(self.frame.height)
                            {
                                self.scanline(
                                    scene, max_step, y, slice_in, slice_out, sample, offset,
                                );
                            }
                        } else {
                            pool.install(|| {
                                back_fb
                                    .buffer_mut()
                                    .par_chunks_mut(self.frame.width)
                                    .zip(read_lock.buffer().par_chunks(self.frame.width))
                                    .enumerate()
                                    .take(self.frame.height)
                                    .for_each(|(y, (slice_out, slice_in))| {
                                        self.scanline(
                                            scene, max_step, y, slice_in, slice_out, sample, offset,
                                        )
                                    })
                            });
                        }
                    }

                    let now = Instant::now();

                    if (now - last_update).as_millis() > 8 {
                        last_update = now;
                        {
                            let mut write_lock = front_fb.write().unwrap();

                            std::mem::swap(&mut back_fb, &mut write_lock);
                        }

                        tx.send(RenderOutMsg::Update(current_scale)).unwrap();
                    }

                    if current_scale != self.scaling {
                        current_scale = current_scale.lower();
                        let (w, h) = (
                            window_size.0 as u32 / current_scale.scale(),
                            window_size.1 as u32 / current_scale.scale(),
                        );

                        self.frame.width = w as usize;
                        self.frame.height = h as usize;

                        sample = 0;
                        continue 'sample;
                    }

                    sample += 1;
                }
            }
        }
    }

    fn msg_to_actions(msg: Result<RenderInMsg, RecvError>) -> RendererActions {
        match msg {
            Err(RecvError::Disconnected) | Ok(RenderInMsg::Exit) => RendererActions::Exit,
            Ok(RenderInMsg::SceneChange(scene)) => RendererActions::Restart {
                scene_change: Some(scene),
                resize_buffers: None,
            },
            Ok(RenderInMsg::Resize(x, y)) => RendererActions::Restart {
                scene_change: None,
                resize_buffers: Some((x, y)),
            },
            Ok(RenderInMsg::Restart) => RendererActions::Restart {
                scene_change: None,
                resize_buffers: None,
            },
        }
    }

    fn scanline(
        &self,
        scene: &Scene,
        max_step: f64,
        y: usize,
        slice_input: &[Pixel],
        slice_output: &mut [Pixel],
        sample: usize,
        offset: (f64, f64),
    ) {
        if let Region::Window { y_min, y_max, .. } = self.frame.region {
            if y >= y_max || y < y_min {
                return;
            }
        }

        let rel_y = (y as f64 + offset.1) / (self.frame.height as f64);

        for (x, pixel) in slice_input.iter().enumerate() {
            if let Region::Window { x_min, x_max, .. } = self.frame.region {
                if x >= x_max || x < x_min {
                    continue;
                }
            }

            let rel_x = (x as f64 + offset.0) / (self.frame.width as f64);

            let sample_info = self.ray_marcher.color_for_ray(
                scene
                    .camera
                    .cast_ray(rel_x, rel_y, self.frame.aspect_ratio()),
                &scene,
                max_step,
                0,
            );

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

pub enum RendererActions {
    Exit,
    Restart {
        resize_buffers: Option<(u32, u32)>,
        scene_change: Option<Scene>,
    },
}

pub enum RenderInMsg {
    Resize(u32, u32),
    SceneChange(Scene),
    Restart,
    Exit,
}

pub enum RenderOutMsg {
    Update(Scaling),
}
