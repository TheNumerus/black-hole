use cgmath::{Array, ElementWise, InnerSpace, Matrix3, Rad, Vector3, VectorSpace, Zero};

use rand::{Rng, SeedableRng};

use blackhole::material::MaterialResult;
use blackhole::math::rand_unit_vector;
use blackhole::shader::{BackgroundShader, SolidShader, VolumetricShader};
use blackhole::texture::{NoiseTexture3D, Texture3D};
use blackhole::BLACKBODY_LUT;
use blackhole::{Ray, RayKind};

pub struct SolidColorShader {
    albedo: Vector3<f64>,
}

impl SolidColorShader {
    pub fn new(albedo: Vector3<f64>) -> Self {
        Self { albedo }
    }
}

impl SolidShader for SolidColorShader {
    fn material_at(&self, ray: &Ray, normal: Vector3<f64>) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: self.albedo,
            emission: Vector3::zero(),
        };

        let dir = rand_unit_vector();

        let mut ray = Ray {
            direction: (normal + dir).normalize(),
            kind: RayKind::Secondary,
            ..*ray
        };
        ray.advance(0.01);

        (mat, Some(ray))
    }
}

pub struct BlackHoleEmitterShader {
    noise: NoiseTexture3D,
}

impl BlackHoleEmitterShader {
    pub fn new() -> Self {
        Self {
            noise: NoiseTexture3D::new(10.0, 0),
        }
    }
}

impl VolumetricShader for BlackHoleEmitterShader {
    fn density_at(&self, position: Vector3<f64>) -> f64 {
        let mag = position.magnitude();
        let noise_coords = {
            let norm = position.normalize();

            let norm_rot = Matrix3::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Rad(mag)) * norm;

            let coords = Vector3::new(norm_rot.x, norm_rot.z, mag);

            coords.mul_element_wise(Vector3::new(1.0, 1.0, 0.1))
        };

        let len_factor = ((4.0 - (mag - 1.0)) / 2.5).min(1.0).max(0.0);

        let noise_factor = self.noise.color_at(noise_coords) * len_factor;

        let noise_factor = if noise_factor > 0.45 { 1.0 } else { 0.0 };

        (0.02 - position.y.abs()) * 100.0 * (4.0 - position.xz().magnitude()) * noise_factor
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let noise_factor = self.noise.color_at(ray.location) * 0.5 + 0.75;

        let temp = (0.02 - ray.location.y.abs())
            * 50.0
            * 2000.0
            * (4.0 - ray.location.xz().magnitude())
            * noise_factor;

        let mat = MaterialResult {
            albedo: Vector3::zero(),
            emission: BLACKBODY_LUT.lookup(temp) * 5.0,
        };

        (mat, None)
    }
}

pub struct VolumeEmitterShader {
    temp: f64,
    density: f64,
    strength: f64,
}

impl VolumeEmitterShader {
    pub fn new(temp: f64, density: f64, strength: f64) -> Self {
        Self {
            temp,
            density,
            strength,
        }
    }
}

impl VolumetricShader for VolumeEmitterShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        self.density
    }

    fn material_at(&self, _ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: Vector3::zero(),
            emission: BLACKBODY_LUT.lookup(self.temp) * self.strength,
        };

        (mat, None)
    }
}

pub struct SolidColorVolumeShader {
    density: f64,
}

impl SolidColorVolumeShader {
    pub fn new(density: f64) -> Self {
        Self { density }
    }
}

impl VolumetricShader for SolidColorVolumeShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        self.density
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: Vector3::from_value(0.8),
            emission: Vector3::zero(),
        };

        let dir = rand_unit_vector();

        let ray = Ray {
            direction: dir,
            kind: RayKind::Secondary,
            ..*ray
        };

        (mat, Some(ray))
    }
}

pub struct BlackHoleScatterShader {
    noise: NoiseTexture3D,
}

impl BlackHoleScatterShader {
    pub fn new() -> Self {
        Self {
            noise: NoiseTexture3D::new(5.0, 0),
        }
    }
}

impl VolumetricShader for BlackHoleScatterShader {
    fn density_at(&self, position: Vector3<f64>) -> f64 {
        let mag = position.magnitude();
        let noise_coords = {
            let norm = position.normalize();

            let norm_rot = Matrix3::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Rad(mag)) * norm;

            let coords = Vector3::new(norm_rot.x, norm_rot.z, mag);

            coords.mul_element_wise(Vector3::new(1.0, 1.0, 0.1))
        };

        let dist_factor = -0.09 * mag.powi(3) + 0.12 * mag.powi(2) + 0.97 * mag - 0.8;

        let noise_factor = 1.0 - self.noise.color_at(noise_coords);

        (0.06 - position.y.abs()) * 100.0 * noise_factor * dist_factor
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: Vector3::new(0.6, 0.6, 0.6),
            emission: Vector3::zero(),
        };

        let dir = rand_unit_vector();

        let ray = Ray {
            direction: dir,
            kind: RayKind::Secondary,
            ..*ray
        };

        (mat, Some(ray))
    }
}

pub struct DebugNoiseVolumeShader {
    noise: NoiseTexture3D,
}

impl DebugNoiseVolumeShader {
    pub fn new() -> Self {
        Self {
            noise: NoiseTexture3D::new(10.0, 0),
        }
    }
}

impl VolumetricShader for DebugNoiseVolumeShader {
    fn density_at(&self, position: Vector3<f64>) -> f64 {
        self.noise.color_at(position).powf(8.0) * 1000.0
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: Vector3::new(0.9, 0.2, 0.1),
            emission: Vector3::zero(),
        };

        let dir = rand_unit_vector();

        let ray = Ray {
            direction: dir,
            kind: RayKind::Secondary,
            ..*ray
        };

        (mat, Some(ray))
    }
}

pub struct SolidColorBackgroundShader {
    color: Vector3<f64>,
}

impl SolidColorBackgroundShader {
    pub fn new(color: Vector3<f64>) -> Self {
        Self { color }
    }
}

impl BackgroundShader for SolidColorBackgroundShader {
    fn emission_at(&self, _ray: &Ray) -> Vector3<f64> {
        self.color
    }
}

pub struct DebugBackgroundShader;

impl BackgroundShader for DebugBackgroundShader {
    fn emission_at(&self, ray: &Ray) -> Vector3<f64> {
        Vector3::new(
            ray.direction.x.max(0.0),
            ray.direction.y.max(0.0),
            ray.direction.z.max(0.0),
        )
    }
}

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
