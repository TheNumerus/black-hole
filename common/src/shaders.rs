use cgmath::{Array, ElementWise, InnerSpace, Matrix3, Rad, Vector3, Zero};

use blackhole::material::MaterialResult;
use blackhole::math::{rand_unit, rand_unit_vector, sigmoid};
use blackhole::shader::{BackgroundShader, Parameter, Shader, VolumetricShader};
use blackhole::texture::{NoiseTexture3D, Texture3D};
use blackhole::BLACKBODY_LUT;
use blackhole::{Ray, RayKind};

mod basic_solid;
mod star_sky;

pub use basic_solid::BasicSolidShader;
pub use star_sky::StarSkyShader;

pub struct BlackHoleEmitterShader {
    noise: NoiseTexture3D,
}

impl BlackHoleEmitterShader {
    pub fn new() -> Self {
        Self {
            noise: NoiseTexture3D::new(10.0, 0, 1),
        }
    }
}

impl Default for BlackHoleEmitterShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for BlackHoleEmitterShader {}

impl VolumetricShader for BlackHoleEmitterShader {
    fn density_at(&self, position: Vector3<f64>) -> f64 {
        let mag = position.magnitude();
        let noise_coords = {
            let norm = position.normalize();

            let norm_rot = Matrix3::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Rad(mag)) * norm;

            let coords = Vector3::new(norm_rot.x, norm_rot.z, mag);

            coords.mul_element_wise(Vector3::new(1.0, 1.0, 0.1))
        };

        let len_factor = (-(2.0 / 5.0) * mag + 2.0).min(20.0 * mag - 20.0);

        let noise_factor = self.noise.color_at(noise_coords) * len_factor;

        let noise_factor = sigmoid(noise_factor, 30.0, 0.52);

        (0.02 - position.y.abs()) * 100.0 * (4.0 - position.xz().magnitude()) * noise_factor
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mag = ray.location.magnitude();
        let noise_coords = {
            let norm = ray.location.normalize();

            let norm_rot = Matrix3::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Rad(mag)) * norm;

            let coords = Vector3::new(norm_rot.x, norm_rot.z, mag);

            coords.mul_element_wise(Vector3::new(1.0, 1.0, 0.1))
        };

        let noise_factor = self.noise.color_at(noise_coords) * 0.5 + 0.75;

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
    pub fn new() -> Self {
        Self {
            temp: 2800.0,
            density: 1.0,
            strength: 1.0,
        }
    }
}

impl Default for VolumeEmitterShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for VolumeEmitterShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("temp", Parameter::Float(f)) => self.temp = f,
            ("density", Parameter::Float(f)) => self.density = f,
            ("strength", Parameter::Float(f)) => self.strength = f,
            _ => {}
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
    albedo: Vector3<f64>,
    density: f64,
}

impl SolidColorVolumeShader {
    pub fn new() -> Self {
        Self {
            albedo: Vector3::from_value(0.8),
            density: 1.0,
        }
    }
}

impl Default for SolidColorVolumeShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for SolidColorVolumeShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("albedo", Parameter::Vec3(v)) => self.albedo = v,
            ("density", Parameter::Float(f)) => self.density = f,
            _ => {}
        }
    }
}

impl VolumetricShader for SolidColorVolumeShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        self.density
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: self.albedo,
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

pub struct SolidColorVolumeAbsorbShader {
    absorption: Vector3<f64>,
    density: f64,
}

impl SolidColorVolumeAbsorbShader {
    pub fn new() -> Self {
        Self {
            absorption: Vector3::from_value(0.8),
            density: 1.0,
        }
    }
}

impl Default for SolidColorVolumeAbsorbShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for SolidColorVolumeAbsorbShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("absorption", Parameter::Vec3(v)) => self.absorption = v,
            ("density", Parameter::Float(f)) => self.density = f,
            _ => {}
        }
    }
}

impl VolumetricShader for SolidColorVolumeAbsorbShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        self.density
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let mat = MaterialResult {
            albedo: self.absorption,
            emission: Vector3::zero(),
        };

        let ray = Ray {
            direction: ray.direction,
            kind: RayKind::Secondary,
            ..*ray
        };

        (mat, Some(ray))
    }
}

pub struct SolidColorVolumeScatterShader {
    scatter: Vector3<f64>,
    absorption: Vector3<f64>,
    density: f64,
}

impl SolidColorVolumeScatterShader {
    pub fn new() -> Self {
        Self {
            scatter: Vector3::from_value(0.8),
            absorption: Vector3::from_value(0.8),
            density: 1.0,
        }
    }
}

impl Default for SolidColorVolumeScatterShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for SolidColorVolumeScatterShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("scatter", Parameter::Vec3(v)) => self.scatter = v,
            ("absorption", Parameter::Vec3(v)) => self.absorption = v,
            ("density", Parameter::Float(f)) => self.density = f,
            _ => {}
        }
    }
}

impl VolumetricShader for SolidColorVolumeScatterShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        self.density
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        let rand = rand_unit();

        //let color_sum = self.scatter.sum() / 3.0;

        if rand > 0.5 {
            let mat = MaterialResult {
                albedo: self.absorption,
                emission: Vector3::zero(),
            };

            let ray = Ray {
                direction: ray.direction,
                kind: RayKind::Secondary,
                ..*ray
            };

            (mat, Some(ray))
        } else {
            let mat = MaterialResult {
                albedo: self.scatter,
                emission: Vector3::zero(),
            };

            let ray = Ray {
                direction: rand_unit_vector(),
                kind: RayKind::Secondary,
                ..*ray
            };

            (mat, Some(ray))
        }
    }
}

pub struct BlackHoleScatterShader {
    noise: NoiseTexture3D,
}

impl BlackHoleScatterShader {
    pub fn new() -> Self {
        Self {
            noise: NoiseTexture3D::new(5.0, 0, 1),
        }
    }
}

impl Default for BlackHoleScatterShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for BlackHoleScatterShader {}

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
            noise: NoiseTexture3D::new(10.0, 0, 1),
        }
    }
}

impl Default for DebugNoiseVolumeShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for DebugNoiseVolumeShader {}

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
    pub fn new() -> Self {
        Self {
            color: Vector3::from_value(0.5),
        }
    }
}

impl Default for SolidColorBackgroundShader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader for SolidColorBackgroundShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("color", Parameter::Vec3(v)) => self.color = v,
            _ => {}
        }
    }
}

impl BackgroundShader for SolidColorBackgroundShader {
    fn emission_at(&self, _ray: &Ray) -> Vector3<f64> {
        self.color
    }
}

pub struct DebugBackgroundShader;

impl Default for DebugBackgroundShader {
    fn default() -> Self {
        Self
    }
}

impl Shader for DebugBackgroundShader {}

impl BackgroundShader for DebugBackgroundShader {
    fn emission_at(&self, ray: &Ray) -> Vector3<f64> {
        Vector3::new(
            ray.direction.x.max(0.0),
            ray.direction.y.max(0.0),
            ray.direction.z.max(0.0),
        )
    }
}
