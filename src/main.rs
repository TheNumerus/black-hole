use std::fs::File;
use std::io::BufWriter;
use std::ops::{Add, AddAssign, Mul};

use cgmath::{Array, InnerSpace, Vector3, Zero};

use clap::{Parser, ValueEnum};

use args::Args;
use camera::Camera;
use object::shape::{Composite, Cylinder, Sphere};
use object::{Distortion, Object};
use scene::Scene;

mod args;
mod camera;
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
    let mut total_steps = 0;

    let offsets_x = [3.0 / 8.0, 7.0 / 8.0, 1.0 / 8.0, 5.0 / 8.0];
    let offsets_y = [1.0 / 8.0, 3.0 / 8.0, 5.0 / 8.0, 7.0 / 8.0];

    let samples = if args.multisampled { 4 } else { 1 };

    for i in 0..samples {
        let mut max_steps_sample = 0;
        for y in 0..args.height {
            for x in 0..args.width {
                let rel_x = (x as f64 + offsets_x[i]) / (args.width as f64);
                let rel_y = (y as f64 + offsets_y[i]) / (args.height as f64);

                let buf_idx = x + y * args.width;

                let sample_info =
                    sample(&scene, max_step, camera.cast_ray(rel_x, rel_y), args.mode);

                max_steps_sample = max_steps_sample.max(sample_info.steps);
                total_steps += sample_info.steps;
                if let RenderMode::Samples = args.mode {
                    buf[buf_idx] += Pixel::new(sample_info.steps as f32, 0.0, 0.0, 0.0);
                } else {
                    let base = buf[buf_idx];

                    buf[buf_idx] = base * (i as f32 / (i as f32 + 1.0))
                        + sample_info.final_color * (1.0 / (i as f32 + 1.0));
                }
            }
        }
        max_step_count += max_steps_sample;
    }

    if let RenderMode::Samples = args.mode {
        for y in 0..args.height {
            for x in 0..args.width {
                let buf_idx = x + y * args.width;

                let sample_count = buf[buf_idx].r;

                let value = sample_count / 1024.0;

                buf[buf_idx] = Pixel::new(value, 1.0 - value, 0.0, 1.0);
            }
        }
    }

    let end = std::time::Instant::now();

    println!("Render took {:.02} seconds", (end - start).as_secs_f64());
    println!("Max steps: {max_step_count}");
    println!(
        "Avg steps per pixel: {}",
        total_steps as f64 / (args.width * args.height) as f64
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
    sphere.radius = 1.0;

    let mut cylinder = Cylinder::new();
    cylinder.height = 0.02;
    cylinder.radius = 3.0;

    let composite = Composite::Diff(Box::new(cylinder), Box::new(sphere));
    let composite = Object::volumetric(Box::new(composite));

    let mut sphere_2 = Sphere::new();
    sphere_2.center = Vector3::new(1.5, 0.0, 0.71);
    sphere_2.radius = 0.2;
    let sphere_2 = Object::solid(Box::new(sphere_2));

    let mut sphere_3 = Sphere::new();
    sphere_3.center = Vector3::new(-2.0, 0.00, -0.81);
    sphere_3.radius = 0.2;
    let sphere_3 = Object::solid(Box::new(sphere_3));

    let mut scene = Scene::new().push(composite).push(sphere_2).push(sphere_3);

    scene.distortions.push(Distortion::new());
    scene
}

fn sample(scene: &Scene, max_step: f64, mut ray: Ray, render_mode: RenderMode) -> Sample {
    let mut pixel = Pixel::black();

    let mut i = 0;
    'pixel: loop {
        let mut dst = f64::MAX;

        let mut can_early_exit = true;
        for distortion in &scene.distortions {
            if !distortion.can_ray_hit(&ray) {
                continue;
            }
            let dist = distortion.dist_fn(ray.location);
            if dist <= 0.0 {
                can_early_exit = false;
            }
            dst = dst.min(dist.max(0.1));
        }

        let mut obj = None;

        for object in &scene.objects {
            if !object.shape.can_ray_hit(&ray) && can_early_exit {
                continue;
            }

            let obj_dist = object.shape.dist_fn(ray.location);
            if obj_dist < dst {
                dst = dst.min(obj_dist);
                obj = Some(object);
            }
        }

        if let Some(obj) = obj {
            if dst < 0.00001 || i == MAX_STEPS {
                let color = get_color(&ray, render_mode, obj, &scene);

                pixel = Pixel::new(color.x as f32, color.y as f32, color.z as f32, 1.0);
                break 'pixel;
            }
        }

        for distortion in &scene.distortions {
            if distortion.is_inside(ray.location) {
                let force = (distortion.shape.center - ray.location).normalize()
                    * dst
                    * distortion.strength(ray.location);

                let new_dir = (ray.direction + force).normalize();

                if ray.direction.dot(new_dir) < -0.9 {
                    break 'pixel;
                }
                ray.direction = new_dir;
            }
        }

        if dst > max_step {
            break;
        }

        if i >= MAX_STEPS {
            break;
        }
        i += 1;

        ray.advance(dst);
    }

    Sample {
        final_color: pixel,
        steps: i,
    }
}

fn get_color(ray: &Ray, render_mode: RenderMode, object: &Object, scene: &Scene) -> Vector3<f64> {
    match render_mode {
        RenderMode::Color => object.shade(scene, ray),
        RenderMode::Normal => {
            let eps = 0.00001;

            object.shape.normal(ray.location, eps) * 0.5 + Vector3::from_value(0.5)
        }
        RenderMode::Shaded => Vector3::from_value(rand::random()),
        // handled for all pixels elsewhere
        RenderMode::Samples => Vector3::zero(),
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

pub struct Ray {
    location: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    pub fn advance(&mut self, dist: f64) {
        self.location += self.direction * dist;
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum RenderMode {
    Samples,
    Normal,
    Color,
    Shaded,
}

pub struct Sample {
    pub(crate) steps: usize,
    pub(crate) final_color: Pixel,
}
