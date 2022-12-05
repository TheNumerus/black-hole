use cgmath::{Array, ElementWise, InnerSpace, Vector3, Zero};

use clap::ValueEnum;

use rand::Rng;

use rayon::prelude::*;

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

use blackhole::filter::{BlackmanHarrisFilter, PixelFilter};
use blackhole::frame::{Frame, Region};
use blackhole::framebuffer::{FrameBuffer, Pixel};
use blackhole::material::MaterialResult;
use blackhole::object::{Object, Shading};
use blackhole::scene::Scene;
use blackhole::Ray;

static TOTAL_STEPS: AtomicUsize = AtomicUsize::new(0);
static MAX_STEPS_PER_SAMPLE: AtomicUsize = AtomicUsize::new(0);

pub struct Renderer {
    pub mode: RenderMode,
    pub samples: usize,
    pub threads: usize,
    pub max_steps: usize,
    pub max_depth: usize,
    pub frame: Frame,
    pub filter: Box<dyn PixelFilter>,
    pub scaling: Scaling,
}

impl Renderer {
    pub fn render(&mut self, scene: &Scene, fb: &mut FrameBuffer) {
        let start = std::time::Instant::now();

        let max_step = scene.max_possible_step(scene.camera.location);

        let mut max_step_count = 0;

        TOTAL_STEPS.store(0, Ordering::SeqCst);

        for i in 0..self.samples {
            let offset = self.filter.next().unwrap();

            if self.threads == 1 {
                for (y, slice) in fb.buffer_mut().chunks_mut(self.frame.width).enumerate() {
                    self.scanline(scene, max_step, y, slice, 0, i, offset);
                }
            } else {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(self.threads)
                    .build()
                    .expect("Failed to build rendering threadpool");

                pool.install(|| {
                    fb.buffer_mut()
                        .par_chunks_mut(self.frame.width)
                        .enumerate()
                        .for_each(|(y, slice)| {
                            self.scanline(scene, max_step, y, slice, 0, i, offset)
                        })
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

        if let RenderMode::Samples = self.mode {
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

    pub fn render_interactive(
        &mut self,
        front_fb: Arc<RwLock<FrameBuffer>>,
        tx: Sender<RenderOutMsg>,
        rx: Receiver<RenderInMsg>,
    ) -> () {
        let mut should_render = false;

        let mut back_fb = FrameBuffer::new(self.frame.width, self.frame.height);

        let mut scene: Option<Scene> = None;

        'main: loop {
            if should_render && scene.is_some() {
                let scene_unwrapped = scene.as_ref().unwrap();
                let max_step = scene_unwrapped.max_possible_step(scene_unwrapped.camera.location);
                for i in 0..self.samples {
                    if let Some(msg) = rx.try_iter().next() {
                        match msg {
                            RenderInMsg::Resize(w, h) => {
                                let (w, h) = match self.scaling {
                                    Scaling::X1 => (w, h),
                                    Scaling::X2 => (w / 2, h / 2),
                                    Scaling::X4 => (w / 4, h / 4),
                                };

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
                            RenderInMsg::Stop => {
                                should_render = false;
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
                                self.scanline_another_target(
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
                            let pool = rayon::ThreadPoolBuilder::new()
                                .num_threads(self.threads)
                                .build()
                                .expect("Failed to build rendering threadpool");

                            pool.install(|| {
                                back_fb
                                    .buffer_mut()
                                    .par_chunks_mut(self.frame.width)
                                    .zip(read_lock.buffer().par_chunks(self.frame.width))
                                    .enumerate()
                                    .for_each(|(y, (slice_out, slice_in))| {
                                        self.scanline_another_target(
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
                    let (w, h) = match self.scaling {
                        Scaling::X1 => (w, h),
                        Scaling::X2 => (w / 2, h / 2),
                        Scaling::X4 => (w / 4, h / 4),
                    };

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
                RenderInMsg::Stop => {
                    should_render = false;
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
        slice: &mut [Pixel],
        slice_start: usize,
        sample: usize,
        offset: (f64, f64),
    ) {
        for (x, pixel) in slice.iter_mut().enumerate() {
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

            let sample_info = self.color_for_ray(
                scene
                    .camera
                    .cast_ray(rel_x, rel_y, self.frame.aspect_ratio()),
                &scene,
                max_step,
                0,
            );

            MAX_STEPS_PER_SAMPLE.fetch_max(sample_info.steps, Ordering::SeqCst);
            TOTAL_STEPS.fetch_add(sample_info.steps, Ordering::SeqCst);
            if let RenderMode::Samples = self.mode {
                *pixel += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
            } else {
                let base = *pixel;

                let color = Pixel::from(sample_info.color);

                *pixel = base * (sample as f32 / (sample as f32 + 1.0))
                    + color * (1.0 / (sample as f32 + 1.0));
            }
        }
    }

    fn scanline_another_target(
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

            let sample_info = self.color_for_ray(
                scene
                    .camera
                    .cast_ray(rel_x, rel_y, self.frame.aspect_ratio()),
                &scene,
                max_step,
                0,
            );

            MAX_STEPS_PER_SAMPLE.fetch_max(sample_info.steps, Ordering::SeqCst);
            TOTAL_STEPS.fetch_add(sample_info.steps, Ordering::SeqCst);
            if let RenderMode::Samples = self.mode {
                slice_output[x] += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
            } else {
                let base = *pixel;

                let color = Pixel::from(sample_info.color);

                slice_output[x] = base * (sample as f32 / (sample as f32 + 1.0))
                    + color * (1.0 / (sample as f32 + 1.0));
            }
        }
    }

    fn color_for_ray(&self, ray: Ray, scene: &Scene, max_step: f64, depth: usize) -> Sample {
        if depth >= self.max_depth {
            return Sample {
                steps: ray.steps_taken,
                color: Vector3::zero(),
            };
        }

        let mut ray = ray;
        let obj = self.march_to_object(&mut ray, &scene, max_step);

        let mat_res = match obj {
            MarchResult::Object(obj) => {
                let (mat, new_ray) = self.get_color(&ray, self.mode, obj);

                match new_ray {
                    Some(new_ray) => {
                        ray = new_ray;
                    }
                    None => {
                        return Sample {
                            steps: ray.steps_taken,
                            color: mat.emission,
                        };
                    }
                }

                mat
            }
            MarchResult::Background(_direction) => MaterialResult {
                emission: scene.background.emission_at(&ray),
                albedo: Vector3::zero(),
            },
            MarchResult::None => MaterialResult::black(),
        };

        let color_reflected = self.color_for_ray(ray, scene, max_step, depth + 1);

        let color = mat_res.emission + mat_res.albedo.mul_element_wise(color_reflected.color);

        return Sample {
            steps: color_reflected.steps,
            color,
        };
    }

    fn march_to_object<'r, 's>(
        &self,
        ray: &'r mut Ray,
        scene: &'s Scene,
        max_step: f64,
    ) -> MarchResult<'s> {
        let mut i = 0;
        let mut active_distortions = Vec::with_capacity(scene.distortions.len());

        loop {
            let mut dst = f64::MAX;

            active_distortions.clear();
            for distortion in &scene.distortions {
                if !distortion.can_ray_hit(&ray) {
                    continue;
                }
                let dist = distortion.dist_fn(ray.location);
                if dist <= 0.0 {
                    active_distortions.push(distortion);
                }
                dst = dst.min(dist.max(0.1));
            }

            let mut obj = None;

            for object in &scene.objects {
                match &object.shading {
                    Shading::Solid(_) => {
                        if !object.shape.can_ray_hit(&ray) && !active_distortions.is_empty() {
                            continue;
                        }

                        let obj_dist = object.shape.dist_fn(ray.location);
                        if obj_dist < dst {
                            dst = dst.min(obj_dist);
                            obj = Some(object);
                        }
                    }
                    Shading::Volumetric(shader) => {
                        let obj_dist = object.shape.dist_fn(ray.location);

                        if obj_dist < 0.0 {
                            dst = dst.min(0.01);
                            let r = rand::thread_rng().gen_range(0.0..1.0);
                            if (shader.density_at(ray.location) * dst) > r {
                                return MarchResult::Object(object);
                            }
                        } else if obj_dist < dst {
                            dst = dst.min(obj_dist.max(0.002));
                        }
                    }
                }
            }

            if let Some(obj) = obj {
                if dst < 0.00001 {
                    return MarchResult::Object(obj);
                }
            }

            for distortion in &active_distortions {
                let strength = distortion.strength(ray.location);

                if strength > 9.0 {
                    return MarchResult::None;
                }

                let force = (distortion.shape.center() - ray.location).normalize() * dst * strength;

                let new_dir = (ray.direction + force).normalize();

                if ray.direction.dot(new_dir) < -0.0 {
                    return MarchResult::None;
                }
                ray.direction = new_dir;
            }

            if dst > max_step {
                return MarchResult::Background(ray.direction);
            }

            if i >= self.max_steps {
                return MarchResult::None;
            }
            i += 1;

            ray.advance(dst);
        }
    }

    fn get_color(
        &self,
        ray: &Ray,
        render_mode: RenderMode,
        object: &Object,
    ) -> (MaterialResult, Option<Ray>) {
        let (mat, new_ray) = object.shade(ray);

        match render_mode {
            RenderMode::Shaded => (mat, new_ray),
            RenderMode::Normal => {
                let eps = 0.00001;
                let normal =
                    object.shape.normal(ray.location, eps) * 0.5 + Vector3::from_value(0.5);

                (
                    MaterialResult {
                        emission: normal,
                        albedo: Vector3::zero(),
                    },
                    new_ray,
                )
            }
            RenderMode::Samples => (
                MaterialResult {
                    emission: Vector3::zero(),
                    albedo: Vector3::zero(),
                },
                new_ray,
            ),
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            mode: RenderMode::Samples,
            samples: 128,
            threads: 0,
            max_steps: 2 << 16,
            max_depth: 16,
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
    Stop,
    Exit,
}

pub enum RenderOutMsg {
    Update,
}

enum MarchResult<'a> {
    Object(&'a Object),
    Background(Vector3<f64>),
    None,
}

pub struct Sample {
    pub(crate) steps: usize,
    pub(crate) color: Vector3<f64>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum RenderMode {
    Samples,
    Normal,
    Shaded,
}

pub enum Scaling {
    X1,
    X2,
    X4,
}

impl Default for Scaling {
    fn default() -> Self {
        Self::X1
    }
}
