use cgmath::{Array, InnerSpace, Vector3, VectorSpace};
use std::fs::File;
use std::io::BufWriter;

mod camera;
mod object;
mod scene;

use crate::object::{Cylinder, Distortion};
use camera::Camera;
use object::Sphere;
use scene::Scene;

pub const MAX_STEPS: usize = 2 << 7;

fn main() {
    const WIDTH: usize = 1280 << 1;
    const HEIGHT: usize = 720 << 1;

    let start = std::time::Instant::now();

    let mut buf = vec![Pixel::black(); WIDTH * HEIGHT];

    let mut sphere = Sphere::new();
    sphere.radius = 0.45;

    let mut sphere_2 = Sphere::new();
    sphere_2.center = Vector3::new(1.5, 0.0, 0.71);
    sphere_2.radius = 0.2;

    let mut sphere_3 = Sphere::new();
    sphere_3.center = Vector3::new(-2.0, 0.00, -0.81);
    sphere_3.radius = 0.2;

    let mut cylinder = Cylinder::new();
    cylinder.height = 0.02;
    cylinder.radius = 3.0;

    let mut scene = Scene::new()
        .push(Box::new(sphere))
        .push(Box::new(sphere_2))
        .push(Box::new(sphere_3))
        .push(Box::new(cylinder));

    scene.distortions.push(Distortion::new());

    let mut camera = Camera::new();
    camera.location = Vector3::new(0.0, 0.54, 10.0);
    camera.hor_fov = 40.0;
    camera.up(Vector3::new(0.1, 1.0, 0.0));
    camera.set_forward(Vector3::new(0.0, -0.01, -1.0));

    let max_step = scene.max_possible_step(camera.location);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let pixel = &mut buf[x + y * WIDTH];

            let rel_x = (x as f64) / (WIDTH as f64);
            let rel_y = (y as f64) / (HEIGHT as f64);

            let final_color = sample(&scene, max_step, camera.cast_ray(rel_x, rel_y));

            *pixel = final_color;
        }
    }

    let end = std::time::Instant::now();

    println!("Render took {:.02} seconds", (end - start).as_secs_f64());

    let buf = unsafe {
        assert_eq!(std::mem::size_of::<Pixel>(), 4 * std::mem::size_of::<u8>());

        let ptr = buf.as_ptr();
        std::slice::from_raw_parts(ptr as *const u8, WIDTH * HEIGHT * 4)
    };

    let file = File::create("out.png").unwrap();
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, WIDTH as u32, HEIGHT as u32);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(buf).unwrap();
}

fn sample(scene: &Scene, max_step: f64, mut ray: Ray) -> Pixel {
    let mut pixel = Pixel::new(0, 0, 0, 255);

    let mut i = 0;
    'pixel: loop {
        let mut dst = f64::MAX;

        let mut can_early_exit = true;
        for distortion in &scene.distortions {
            let dist = distortion.dist_fn(ray.location);
            if dist <= 0.0 {
                can_early_exit = false;
            }
            dst = dst.min(dist.max(0.1));
        }

        for object in &scene.objects {
            if !object.can_ray_hit(&ray) && can_early_exit {
                continue;
            }

            let obj_dist = object.dist_fn(ray.location);
            if obj_dist <= dst {
                dst = obj_dist;
            }

            if dst < 0.0001 || i == MAX_STEPS {
                let color = object.color(ray.location);
                pixel = Pixel::new(
                    (color.x * 255.0) as u8,
                    (color.y * 255.0) as u8,
                    (color.z * 255.0) as u8,
                    255,
                );

                let normal = {
                    let dist_x = object.dist_fn(ray.location + Vector3::new(0.001, 0.0, 0.0));
                    let dist_y = object.dist_fn(ray.location + Vector3::new(0.0, 0.001, 0.0));
                    let dist_z = object.dist_fn(ray.location + Vector3::new(0.0, 0.0, 0.001));

                    (Vector3::new(dist_x, dist_y, dist_z) - Vector3::from_value(dst)) / 0.001
                };

                pixel = Pixel::new(
                    ((normal.x * 0.5 + 0.5) * 255.0) as u8,
                    ((normal.y * 0.5 + 0.5) * 255.0) as u8,
                    ((normal.z * 0.5 + 0.5) * 255.0) as u8,
                    255,
                );

                break 'pixel;
            }
        }

        for distortion in &scene.distortions {
            if distortion.is_inside(ray.location) {
                dst = dst.min(0.1);
                ray.direction = ray
                    .direction
                    .lerp(
                        (distortion.center - ray.location).normalize(),
                        distortion.strength(ray.location) * dst,
                    )
                    .normalize();
            }
        }

        if dst > max_step {
            break;
        }

        if i > MAX_STEPS {
            break;
        }
        i += 1;

        ray.advance(dst);
    }

    //pixel.r = ((i as f32 / MAX_STEPS as f32) * 255.0) as u8;
    //pixel.g = ((i as f32 / MAX_STEPS as f32) * 240.0) as u8;
    //pixel.b = ((i as f32 / MAX_STEPS as f32) * 180.0) as u8;

    pixel
}

#[derive(Copy, Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
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
