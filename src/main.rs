use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rayon::prelude::*;

use cgmath::{Array, ElementWise, InnerSpace, Vector3, Zero};

use clap::{Parser, ValueEnum};

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::material::MaterialResult;
use crate::object::Shading;
use crate::shader::{
    BlackHoleEmitterShader, BlackHoleScatterShader, SolidColorShader, StarSkyShader,
};

use args::Args;
use camera::Camera;
use framebuffer::{FrameBuffer, Pixel};
use object::shape::{Composite, Cylinder, Sphere};
use object::{Distortion, Object};
use scene::Scene;

mod args;
mod camera;
mod framebuffer;
mod material;
mod object;
mod scene;
mod shader;

pub const MAX_DEPTH: usize = 8;
pub const MAX_STEPS: usize = 2 << 16;

fn main() {
    // clion needs help in trait annotation
    let args = <Args as Parser>::parse();

    let start = std::time::Instant::now();

    let mut fb = FrameBuffer::new(args.width, args.height);

    let scene = setup_scene();
    let camera = setup_camera(args.width as f64, args.height as f64);

    let max_step = scene.max_possible_step(camera.location);

    let mut max_step_count = 0;
    let total_steps = AtomicUsize::new(0);

    let mut sampler = PixelFilter::new(1.5);

    for i in 0..args.samples {
        let offset = sampler.next().unwrap();

        let max_steps_sample = AtomicUsize::new(0);

        fb.buffer_mut()
            .par_chunks_mut(args.width)
            .enumerate()
            .for_each(|(y, slice)| {
                for (x, pixel) in slice.iter_mut().enumerate() {
                    let rel_x = (x as f64 + offset.0) / (args.width as f64);
                    let rel_y = (y as f64 + offset.1) / (args.height as f64);

                    let sample_info = color_for_ray(
                        camera.cast_ray(rel_x, rel_y),
                        &scene,
                        args.mode,
                        max_step,
                        0,
                    );

                    max_steps_sample.fetch_max(sample_info.steps, Ordering::SeqCst);
                    total_steps.fetch_add(sample_info.steps, Ordering::SeqCst);
                    if let RenderMode::Samples = args.mode {
                        *pixel += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
                    } else {
                        let base = *pixel;

                        let color = Pixel::from(sample_info.color);

                        *pixel =
                            base * (i as f32 / (i as f32 + 1.0)) + color * (1.0 / (i as f32 + 1.0));
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
                let pixel = fb.pixel_mut(x, y).unwrap();

                let sample_count = pixel.r;

                let value = sample_count / 256.0 as f32 / args.samples as f32;

                *pixel = Pixel::new(value, 1.0 - value, 0.0, 1.0);
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

    write_out(fb, args.width as u32, args.height as u32);
}

fn write_out(fb: FrameBuffer, width: u32, height: u32) {
    let buf = unsafe {
        assert_eq!(std::mem::size_of::<Pixel>(), 4 * std::mem::size_of::<f32>());

        fb.as_f32_slice()
    };

    let mapped = buf
        .iter()
        .map(|e| (e.powf(1.0 / 2.2) * 255.0) as u8)
        .collect::<Vec<_>>();

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
    cylinder.set_height(0.02);
    cylinder.set_radius(3.0);

    let mut cylinder_scatter = Cylinder::new();
    cylinder_scatter.set_height(0.06);
    cylinder_scatter.set_radius(3.2);

    let bhes = Arc::new(BlackHoleEmitterShader);
    let bhss = Arc::new(BlackHoleScatterShader);
    let asteroid_shader = Arc::new(SolidColorShader::new(Vector3::from_value(0.6)));

    let composite = Composite::diff(Box::new(cylinder), Box::new(sphere.clone()));
    let composite = Object::volumetric(Box::new(composite), bhes);

    let composite_2 = Composite::diff(Box::new(cylinder_scatter), Box::new(sphere));
    let composite_2 = Object::volumetric(Box::new(composite_2), bhss);

    let mut sphere_2 = Sphere::new();
    sphere_2.set_center(Vector3::new(1.5, 0.0, 0.71));
    sphere_2.set_radius(0.2);
    let sphere_2 = Object::solid(Box::new(sphere_2), asteroid_shader.clone());

    let mut sphere_3 = Sphere::new();
    sphere_3.set_center(Vector3::new(-2.0, 0.00, -0.81));
    sphere_3.set_radius(0.2);
    let sphere_3 = Object::solid(Box::new(sphere_3), asteroid_shader.clone());

    let mut scene = Scene::new()
        .push(composite)
        .push(sphere_2)
        .push(sphere_3)
        .push(composite_2);

    scene.distortions.push(Distortion::new());
    scene.set_background(Box::new(StarSkyShader::new(
        42000,
        Vector3::new(0.06, 0.02, 0.3) * 0.03,
    )));
    scene
}

fn color_for_ray(
    ray: Ray,
    scene: &Scene,
    render_mode: RenderMode,
    max_step: f64,
    depth: usize,
) -> Sample {
    if depth >= MAX_DEPTH {
        return Sample {
            steps: ray.steps_taken,
            color: Vector3::zero(),
        };
    }

    let mut ray = ray;
    let obj = march_to_object(&mut ray, &scene, max_step);

    let mat_res = match obj {
        MarchResult::Object(obj) => {
            let (mat, new_ray) = get_color(&ray, render_mode, obj);

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
        MarchResult::Background(direction) => MaterialResult {
            emission: scene.background.emission_at(direction),
            albedo: Vector3::zero(),
        },
        MarchResult::None => MaterialResult::black(),
    };

    let color_reflected = color_for_ray(ray, scene, render_mode, max_step, depth + 1);

    let color = mat_res.emission + mat_res.albedo.mul_element_wise(color_reflected.color);

    return Sample {
        steps: color_reflected.steps,
        color,
    };
}

enum MarchResult<'a> {
    Object(&'a Object),
    Background(Vector3<f64>),
    None,
}

fn march_to_object<'r, 's>(ray: &'r mut Ray, scene: &'s Scene, max_step: f64) -> MarchResult<'s> {
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
                        dst = dst.min(obj_dist.max(0.01));
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
            let force = (distortion.shape.center() - ray.location).normalize()
                * dst
                * distortion.strength(ray.location);

            let new_dir = (ray.direction + force).normalize();

            if ray.direction.dot(new_dir) < -0.9 {
                return MarchResult::None;
            }
            ray.direction = new_dir;
        }

        if dst > max_step {
            return MarchResult::Background(ray.direction);
        }

        if i >= MAX_STEPS {
            return MarchResult::None;
        }
        i += 1;

        ray.advance(dst);
    }
}

fn get_color(ray: &Ray, render_mode: RenderMode, object: &Object) -> (MaterialResult, Option<Ray>) {
    match render_mode {
        RenderMode::Shaded => object.shade(ray),
        RenderMode::Normal => {
            let eps = 0.00001;

            let normal = object.shape.normal(ray.location, eps) * 0.5 + Vector3::from_value(0.5);

            let (_, ray) = object.shade(ray);

            (
                MaterialResult {
                    emission: normal,
                    albedo: Vector3::zero(),
                },
                ray,
            )
        }
        RenderMode::Samples => {
            let (_, ray) = object.shade(ray);
            (
                MaterialResult {
                    emission: Vector3::zero(),
                    albedo: Vector3::zero(),
                },
                ray,
            )
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

    pub fn reflect(&self, normal: Vector3<f64>) -> Self {
        Ray {
            location: self.location,
            direction: self.direction - 2.0 * self.direction.dot(normal) * normal,
            steps_taken: 0,
        }
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
    pub(crate) color: Vector3<f64>,
}

///
/// Sub pixel sampler with Box window-function
///
pub struct PixelFilter {
    pub(crate) generator: SmallRng,
    first_sample: bool,
    filter_size: f64,
}

impl PixelFilter {
    pub fn new(filter_size: f64) -> Self {
        let generator = rand::rngs::SmallRng::seed_from_u64(0);

        Self {
            generator,
            first_sample: true,
            filter_size,
        }
    }
}

impl Iterator for PixelFilter {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_sample {
            let range = -(self.filter_size / 2.0)..(self.filter_size / 2.0);

            let x = self.generator.gen_range(range.clone());
            let y = self.generator.gen_range(range);

            Some((x + 0.5, y + 0.5))
        } else {
            Some((0.5, 0.5))
        }
    }
}
