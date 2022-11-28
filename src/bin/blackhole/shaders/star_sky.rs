use blackhole::math::rand_unit_vector;
use blackhole::shader::BackgroundShader;
use blackhole::{Ray, RayKind};

use cgmath::{Array, ElementWise, InnerSpace, Vector3, VectorSpace, Zero};

use rand::{Rng, SeedableRng};

#[derive(Debug, Clone)]
struct Star {
    direction: Vector3<f64>,
    color: Vector3<f64>,
    brightness: f64,
}

pub struct StarSkyShader {
    stars: Vec<Vec<Star>>,
    star_x_divisions: usize,
    star_y_divisions: usize,
    milky_way_color: Vector3<f64>,
}

impl StarSkyShader {
    pub fn new(star_count: usize, milky_way_color: Vector3<f64>) -> Self {
        let star_x_divisions = 128;
        let star_y_divisions = 64;

        let mut stars = vec![Vec::new(); star_x_divisions * star_y_divisions];

        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

        for _star_index in 0..star_count {
            let dir = rand_unit_vector();

            let (x, y) = Self::sector_from_dir(star_x_divisions, star_y_divisions, &dir);

            let color_scale = rng.gen_range(0.0_f64..1.0).powf(2.0);

            let color = Vector3::new(0.9, 0.6, 0.2).lerp(Vector3::new(0.6, 0.8, 1.0), color_scale);
            let brightness = color_scale * 0.8 + 0.1;

            let star = Star {
                direction: dir,
                color,
                brightness,
            };

            stars[x + y * star_x_divisions].push(star);
        }

        Self {
            stars,
            milky_way_color,
            star_x_divisions,
            star_y_divisions,
        }
    }

    fn sector_from_dir(
        star_x_divisions: usize,
        star_y_divisions: usize,
        dir: &Vector3<f64>,
    ) -> (usize, usize) {
        let xz = dir.xz().normalize();

        let x = if xz.x > 0.0 {
            ((((xz.y + 1.0) / 2.0) * star_x_divisions as f64 / 2.0) as usize)
                .min((star_x_divisions - 1) / 2)
        } else {
            ((((xz.y + 1.0) / 2.0) * star_x_divisions as f64 / 2.0) as usize)
                .min((star_x_divisions - 1) / 2)
                + star_x_divisions / 2
        };

        let y = (((dir.y + 1.0) / 2.0) * star_y_divisions as f64).floor() as usize;

        (x, y)
    }
}

impl BackgroundShader for StarSkyShader {
    fn emission_at(&self, ray: &Ray) -> Vector3<f64> {
        let mut color = Vector3::zero();

        if let RayKind::Primary = ray.kind {
            let (x, y) =
                Self::sector_from_dir(self.star_x_divisions, self.star_y_divisions, &ray.direction);

            for x_sector in (x as i32 - 1)..=(x as i32 + 1) {
                for y_sector in (y as i32 - 1)..=(y as i32 + 1) {
                    let x_sector =
                        (x_sector + self.star_x_divisions as i32) as usize % self.star_x_divisions;
                    let y_sector = (y_sector.max(0) as usize).min(self.star_y_divisions - 1);

                    for star in &self.stars[x_sector + y_sector * self.star_x_divisions] {
                        let dot = star.direction.dot(ray.direction);

                        let pow = (2.0 - star.brightness) * 8_000_000.0;

                        if dot > 0.999999 {
                            color += Vector3::from_value(dot.powf(pow))
                                .mul_element_wise(star.color)
                                * star.brightness;
                        }
                    }
                }
            }
        }

        color += std::f64::consts::E.powf(-100.0 * ray.direction.y.powi(2)) * self.milky_way_color;

        color
    }
}
