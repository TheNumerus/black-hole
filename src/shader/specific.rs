use cgmath::{Array, ElementWise, InnerSpace, Vector3, VectorSpace, Zero};

use crate::lut::LookupTable;
use rand::{Rng, SeedableRng};

use crate::material::MaterialResult;
use crate::math::rand_unit_vector;
use crate::shader::{BackgroundShader, SolidShader, VolumetricShader};
use crate::texture::{NoiseTexture3D, Texture3D};
use crate::Ray;

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
            ..*ray
        };
        ray.advance(0.01);

        (mat, Some(ray))
    }
}

pub struct BlackHoleEmitterShader {
    bb_lut: LookupTable<Vector3<f64>>,
}

impl BlackHoleEmitterShader {
    pub fn new() -> Self {
        Self {
            bb_lut: blackbody_lookup(),
        }
    }
}

impl VolumetricShader for BlackHoleEmitterShader {
    fn density_at(&self, position: Vector3<f64>) -> f64 {
        (0.02 - position.y.abs()) * 100.0 * (4.0 - position.xz().magnitude())
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let temp =
            (0.02 - ray.location.y.abs()) * 50.0 * 2000.0 * (4.0 - ray.location.xz().magnitude());

        let mat = MaterialResult {
            albedo: Vector3::zero(),
            emission: self.bb_lut.lookup(temp) * 5.0,
        };

        (mat, None)
    }
}

pub struct VolumeEmitterShader {
    temp: f64,
    density: f64,
    strength: f64,
    bb_lut: LookupTable<Vector3<f64>>,
}

impl VolumeEmitterShader {
    pub fn new(temp: f64, density: f64, strength: f64) -> Self {
        Self {
            temp,
            density,
            strength,
            bb_lut: blackbody_lookup(),
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
            emission: self.bb_lut.lookup(self.temp) * self.strength,
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
            ..*ray
        };

        (mat, Some(ray))
    }
}

pub struct BlackHoleScatterShader;

impl VolumetricShader for BlackHoleScatterShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        (0.06 - _position.y.abs()) * 100.0
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: Vector3::new(0.6, 0.6, 0.6),
            emission: Vector3::zero(),
        };

        let dir = rand_unit_vector();

        let ray = Ray {
            direction: dir,
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
            noise: NoiseTexture3D::new(10.0),
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
    fn emission_at(&self, _direction: Vector3<f64>) -> Vector3<f64> {
        self.color
    }
}

pub struct DebugBackgroundShader;

impl BackgroundShader for DebugBackgroundShader {
    fn emission_at(&self, direction: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            direction.x.max(0.0),
            direction.y.max(0.0),
            direction.z.max(0.0),
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
    fn emission_at(&self, direction: Vector3<f64>) -> Vector3<f64> {
        let (x, y) =
            Self::sector_from_dir(self.star_x_divisions, self.star_y_divisions, &direction);

        let mut color = Vector3::zero();

        for x_sector in (x as i32 - 1)..=(x as i32 + 1) {
            for y_sector in (y as i32 - 1)..=(y as i32 + 1) {
                let x_sector =
                    (x_sector + self.star_x_divisions as i32) as usize % self.star_x_divisions;
                let y_sector = (y_sector.max(0) as usize).min(self.star_y_divisions - 1);

                for star in &self.stars[x_sector + y_sector * self.star_x_divisions] {
                    let dot = star.direction.dot(direction);

                    let pow = (2.0 - star.brightness) * 8_000_000.0;

                    if dot > 0.999999 {
                        color += Vector3::from_value(dot.powf(pow)).mul_element_wise(star.color)
                            * star.brightness;
                    }
                }
            }
        }

        color += std::f64::consts::E.powf(-100.0 * direction.y.powi(2)) * self.milky_way_color;

        color
    }
}

fn blackbody_lookup() -> LookupTable<Vector3<f64>> {
    LookupTable::from_vec(vec![
        (500.0, Vector3::new(0.0, 0.0, 0.0)),
        (1000.0, Vector3::new(1.0, 0.0, 0.0)),
        (2000.0, Vector3::new(1.0, 0.2, 0.0)),
        (3000.0, Vector3::new(1.0, 0.8, 0.2)),
        (6500.0, Vector3::new(1.0, 1.0, 1.0)),
    ])
}
