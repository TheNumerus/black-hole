use cgmath::Vector3;
use std::fs::File;
use std::io::BufWriter;

mod camera;
mod object;
mod scene;

use camera::Camera;
use object::Sphere;
use scene::Scene;

fn main() {
    const WIDTH: usize = 1280;
    const HEIGHT: usize = 720;

    let mut buf = Box::new([Pixel::black(); WIDTH * HEIGHT]);

    let mut sphere_2 = Sphere::new();
    sphere_2.center = Vector3::new(1.5, 0.0, 0.71);
    sphere_2.radius = 0.7;
    let scene = Scene::new()
        .push(Box::new(Sphere::new()))
        .push(Box::new(sphere_2));

    let mut camera = Camera::new();
    camera.location = Vector3::new(0.0, 3.0, 3.0);
    camera.set_forward(Vector3::new(0.0, -1.0, -1.0));

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let pixel = &mut buf[x + y * WIDTH];

            let rel_x = (x as f32) / (WIDTH as f32);
            let rel_y = (y as f32) / (HEIGHT as f32);

            let mut ray = camera.cast_ray(rel_x, rel_y);

            //let mut final_color = Pixel::new((x % 255) as u8, (y % 255) as u8, 0, 255);
            let mut final_color = Pixel::new(0, 0, 0, 255);

            let mut i = 0;
            'pixel: loop {
                let mut dst = f32::MAX;

                for object in &scene.objects {
                    if !object.can_ray_hit(&ray) {
                        continue;
                    }

                    let obj_dist = object.dist_fn(ray.location);
                    if obj_dist <= dst {
                        dst = obj_dist;
                    }

                    if dst < 0.1 {
                        let color = object.color(ray.location);
                        final_color = Pixel::new(
                            (color.x * 255.0) as u8,
                            (color.y * 255.0) as u8,
                            (color.z * 255.0) as u8,
                            255,
                        );
                        break 'pixel;
                    }
                }

                if i > 64 {
                    break;
                }
                i += 1;

                ray.advance(dst);
            }

            *pixel = final_color;
            pixel.r = ((i as f32 / 64.0) * 255.0) as u8;
            pixel.g = ((i as f32 / 64.0) * 255.0) as u8;
            pixel.b = ((i as f32 / 64.0) * 255.0) as u8;
        }
    }

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

struct Ray {
    location: Vector3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    pub fn advance(&mut self, dist: f32) {
        self.location += self.direction * dist;
    }
}
