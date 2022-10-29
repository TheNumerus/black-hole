use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::{Add, AddAssign, Mul};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rayon::prelude::*;

use cgmath::{Array, InnerSpace, Vector3, Zero};

use clap::{Parser, ValueEnum};

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::light::Light;
use crate::object::Shading;
use args::Args;
use camera::Camera;
use object::shape::{Composite, Cylinder, Sphere};
use object::{Distortion, Object};
use scene::Scene;

mod args;
mod camera;
mod light;
mod object;
mod scene;

pub const MAX_STEPS: usize = 2 << 16;

fn main() {
    // clion needs help in trait annotation
    let args = <Args as Parser>::parse();

    let start = std::time::Instant::now();

    let buf = vec![Pixel::black(); args.width * args.height].leak();

    let scene = setup_scene();
    let camera = setup_camera(args.width as f64, args.height as f64);

    let max_step = scene.max_possible_step(camera.location);

    let mut max_step_count = 0;
    let total_steps = AtomicUsize::new(0);

    let mut sampler = Sampler::new();

    let thread_num = std::thread::available_parallelism()
        .unwrap_or(std::num::NonZeroUsize::new(1).unwrap())
        .get();

    for i in 0..args.samples {
        let offset = sampler.next().unwrap();

        let max_steps_sample = AtomicUsize::new(0);

        buf.par_chunks_mut(args.width)
            .enumerate()
            .for_each(|(y, slice)| {
                for (x, pixel) in slice.iter_mut().enumerate() {
                    let rel_x = (x as f64 + offset.0) / (args.width as f64);
                    let rel_y = (y as f64 + offset.1) / (args.height as f64);

                    let sample_info =
                        sample(&scene, max_step, camera.cast_ray(rel_x, rel_y), args.mode);

                    max_steps_sample.fetch_max(sample_info.steps, Ordering::SeqCst);
                    total_steps.fetch_add(sample_info.steps, Ordering::SeqCst);
                    if let RenderMode::Samples = args.mode {
                        *pixel += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
                    } else {
                        let base = *pixel;

                        *pixel = base * (i as f32 / (i as f32 + 1.0))
                            + sample_info.final_color * (1.0 / (i as f32 + 1.0));
                    }
                }
            });
        max_step_count += max_steps_sample.load(Ordering::SeqCst);

        let sample_end = std::time::Instant::now();
        let remaining_part = args.samples as f32 / (i as f32 + 1.0) - 1.0;
        let time = sample_end - start;
        let remaining_time = time.mul_f32(remaining_part);
        print!(
            "\rSample {}/{}, time: {:02}:{:02}, remaining: {:02}:{:02}",
            i + 1,
            args.samples,
            time.as_secs() / 60,
            time.as_secs() % 60,
            remaining_time.as_secs() / 60,
            remaining_time.as_secs() % 60
        );
        std::io::stdout().flush().expect("Failed to flush stdout");
    }

    println!();

    if let RenderMode::Samples = args.mode {
        for y in 0..args.height {
            for x in 0..args.width {
                let buf_idx = x + y * args.width;

                let sample_count = buf[buf_idx].r;

                let value = sample_count / 128.0 as f32 / args.samples as f32;

                buf[buf_idx] = Pixel::new(value, 1.0 - value, 0.0, 1.0);
            }
        }
    }

    let end = std::time::Instant::now();

    println!("Render took {:.02} seconds", (end - start).as_secs_f64());
    println!("Max steps: {max_step_count}");
    println!(
        "Avg steps per pixel: {}",
        total_steps.load(Ordering::SeqCst) as f64 / (args.width * args.height) as f64
    );

    write_out(buf, args.width as u32, args.height as u32);
}

fn write_out(buf: &mut [Pixel], width: u32, height: u32) {
    let buf = unsafe {
        assert_eq!(std::mem::size_of::<Pixel>(), 4 * std::mem::size_of::<f32>());

        let ptr = buf.as_ptr();
        std::slice::from_raw_parts(ptr as *const f32, buf.len() * 4)
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
    camera.hor_fov = 40.0;
    camera.up(Vector3::new(0.1, 1.0, 0.0));
    camera.set_forward(Vector3::new(0.0, -0.01, -1.0));
    camera.aspect_ratio = width / height;
    camera
}

fn setup_scene() -> Scene {
    let mut sphere = Sphere::new();
    sphere.set_radius(1.0);

    let mut cylinder = Cylinder::new();
    cylinder.set_height(0.04);
    cylinder.set_radius(3.0);

    let composite = Composite::diff(Box::new(cylinder), Box::new(sphere));
    let composite = Object::volumetric(Box::new(composite));

    let mut sphere_2 = Sphere::new();
    sphere_2.set_center(Vector3::new(1.5, 0.0, 0.71));
    sphere_2.set_radius(0.2);
    let sphere_2 = Object::solid(Box::new(sphere_2));

    let mut sphere_3 = Sphere::new();
    sphere_3.set_center(Vector3::new(-2.0, 0.00, -0.81));
    sphere_3.set_radius(0.2);
    let sphere_3 = Object::solid(Box::new(sphere_3));

    let light = Light {
        color: Vector3::new(1.0, 0.8, 0.2),
        location: Vector3::zero(),
        strength: 1.0,
    };

    let mut scene = Scene::new().push(composite).push(sphere_2).push(sphere_3);

    scene.distortions.push(Distortion::new());
    scene.lights.push(light);
    scene
}

fn sample(scene: &Scene, max_step: f64, mut ray: Ray, render_mode: RenderMode) -> Sample {
    let mut pixel = Pixel::black();

    let obj = march_to_object(&mut ray, &scene, max_step);

    if let Some(obj) = obj {
        let color = get_color(&mut ray, render_mode, obj, &scene);

        pixel = Pixel::new(color.x as f32, color.y as f32, color.z as f32, 1.0);
    }

    Sample {
        final_color: pixel,
        steps: ray.steps_taken,
    }
}

fn march_to_object<'r, 's>(
    ray: &'r mut Ray,
    scene: &'s Scene,
    max_step: f64,
) -> Option<&'s Object> {
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
            match object.shading {
                Shading::Solid => {
                    if !object.shape.can_ray_hit(&ray) && !active_distortions.is_empty() {
                        continue;
                    }

                    let obj_dist = object.shape.dist_fn(ray.location);
                    if obj_dist < dst {
                        dst = dst.min(obj_dist);
                        obj = Some(object);
                    }
                }
                Shading::Volumetric { density, .. } => {
                    let obj_dist = object.shape.dist_fn(ray.location);

                    if obj_dist < 0.0 {
                        dst = dst.min(0.01);
                        let r = rand::thread_rng().gen_range(0.0..1.0);
                        if (density * dst) > r {
                            return Some(object);
                        }
                    } else if obj_dist < dst {
                        dst = dst.min(obj_dist.max(0.01));
                    }
                }
            }
        }

        if let Some(obj) = obj {
            if dst < 0.00001 {
                return Some(obj);
            }
        }

        for distortion in &active_distortions {
            let force = (distortion.shape.center() - ray.location).normalize()
                * dst
                * distortion.strength(ray.location);

            let new_dir = (ray.direction + force).normalize();

            if ray.direction.dot(new_dir) < -0.9 {
                return None;
            }
            ray.direction = new_dir;
        }

        if dst > max_step {
            return None;
        }

        if i >= MAX_STEPS {
            return None;
        }
        i += 1;

        ray.advance(dst);
    }
}

fn get_color(
    ray: &mut Ray,
    render_mode: RenderMode,
    object: &Object,
    scene: &Scene,
) -> Vector3<f64> {
    match render_mode {
        RenderMode::Shaded => object.shade(scene, ray),
        RenderMode::Normal => {
            let eps = 0.00001;

            object.shape.normal(ray.location, eps) * 0.5 + Vector3::from_value(0.5)
        }
        RenderMode::Samples => {
            object.shade(scene, ray);
            // handled for all pixels elsewhere
            Vector3::zero()
        }
    }
}

#[derive(Copy, Clone)]
struct Pixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Pixel {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl Add<Pixel> for Pixel {
    type Output = Pixel;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl AddAssign for Pixel {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += 0.0;
    }
}

impl Mul<f32> for Pixel {
    type Output = Pixel;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    location: Vector3<f64>,
    direction: Vector3<f64>,
    steps_taken: usize,
}

impl Ray {
    pub fn advance(&mut self, dist: f64) {
        self.location += self.direction * dist;
        self.steps_taken += 1;
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum RenderMode {
    Samples,
    Normal,
    Shaded,
}

pub struct Sample {
    pub(crate) steps: usize,
    pub(crate) final_color: Pixel,
}

pub struct Sampler {
    pub(crate) generator: SmallRng,
}

impl Sampler {
    pub fn new() -> Self {
        let generator = rand::rngs::SmallRng::seed_from_u64(0);

        Self { generator }
    }
}

impl Iterator for Sampler {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.generator.gen_range(0.0..1.0);
        let y = self.generator.gen_range(0.0..1.0);

        Some((x, y))
    }
}
