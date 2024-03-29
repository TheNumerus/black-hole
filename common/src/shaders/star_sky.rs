use blackhole::math::{rand_unit, rand_unit_vector};
use blackhole::shader::{BackgroundShader, Parameter, Shader};
use blackhole::{Ray, RayKind};

use cgmath::{Array, ElementWise, InnerSpace, Vector3, VectorSpace, Zero};

use blackhole::texture::{NoiseTexture3D, Texture3D, WorleyTexture3D};

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
    noise: NoiseTexture3D,
    worley: WorleyTexture3D,
}

impl StarSkyShader {
    pub fn new() -> Self {
        let mut shader = Self {
            stars: Vec::new(),
            milky_way_color: Vector3::new(0.2, 0.3, 0.4),
            star_x_divisions: 256,
            star_y_divisions: 128,
            noise: NoiseTexture3D::new(20.0, 0, 4),
            worley: WorleyTexture3D::new(8.0),
        };

        shader.regenerate_stars(10_000);

        shader
    }

    fn regenerate_stars(&mut self, new_stars: usize) {
        let mut stars = vec![Vec::new(); self.star_x_divisions * self.star_y_divisions];

        for _star_index in 0..new_stars {
            let dir = rand_unit_vector()
                .mul_element_wise(Vector3::new(3.0, 1.0, 3.0))
                .normalize();

            let (x, y) = Self::sector_from_dir(self.star_x_divisions, self.star_y_divisions, &dir);

            let color_scale = rand_unit().powf(2.0);

            let color = Vector3::new(0.9, 0.6, 0.2).lerp(Vector3::new(0.6, 0.8, 1.0), color_scale);
            let brightness = color_scale * 0.8 + 0.1;

            let star = Star {
                direction: dir,
                color,
                brightness,
            };

            stars[x + y * self.star_x_divisions].push(star);
        }

        self.stars = stars;
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

impl Default for StarSkyShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for StarSkyShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("milky_way_color", Parameter::Vec3(c)) => self.milky_way_color = c,
            ("star_count", Parameter::Usize(c)) => self.regenerate_stars(c),
            _ => {}
        }
    }
}

impl BackgroundShader for StarSkyShader {
    fn emission_at(&self, ray: &Ray) -> Vector3<f64> {
        let mut color = Vector3::zero();

        let noise_factor = {
            let a = self.noise.color_at(ray.direction);
            let b = self.noise.color_at(ray.direction / 3.0);

            let mut value = b.powf(a * 4.0);

            let worley = self
                .worley
                .color_at(ray.direction.add_element_wise(a * 0.1));

            let horizon_factor =
                (1.0 - ray.direction.dot(Vector3::new(0.0, 1.0, 0.0)).abs()).powi(6) + 0.5;

            value += horizon_factor;

            value *= worley * horizon_factor;

            let dot_factor = (ray.direction.dot(Vector3::new(0.0, 0.0, -1.0)) * 0.4) + 1.0;

            value *= dot_factor;

            value = ((value - 0.05) * 1.3).max(0.0);

            value = value.powi(2);

            value
        };

        let (glow_factor, glow_factor_2) = {
            let base = 1.0 - ray.direction.dot(Vector3::new(0.0, 1.0, 0.0)).abs();

            let neg_z_dot = ray.direction.dot(Vector3::new(0.0, 0.0, -1.0));

            let first = neg_z_dot * 0.9 + 1.0;
            let second = neg_z_dot * 0.8 + 1.0;

            ((base * first).powi(2), (base * second).powi(2))
        };

        let glow_first = glow_factor
            * (Vector3::new(0.3, 0.2, 0.0) * 0.04)
            * std::f64::consts::E.powf(-1000.0 * ray.direction.y.powi(2));

        let glow_second = glow_factor_2
            * ((Vector3::new(0.2, 0.1, 0.3) * 0.02)
                * std::f64::consts::E.powf(-400.0 * ray.direction.y.powi(2)))
            + ((Vector3::new(0.1, 0.2, 0.3) * 0.01)
                * std::f64::consts::E.powf(-100.0 * ray.direction.y.powi(2)));

        color += (glow_first + glow_second) * 0.1;

        color += self.milky_way_color * noise_factor * 0.1;

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

        color
    }
}
