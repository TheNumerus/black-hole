use cgmath::{InnerSpace, Vector3, Zero};

use rand::Rng;

use crate::material::MaterialResult;
use crate::shader::{BackgroundShader, SolidShader, VolumetricShader};
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
    fn material_at(&self, ray: &Ray, normal: Vector3<f64>) -> (MaterialResult, Ray) {
        let mat = MaterialResult {
            albedo: self.albedo,
            emission: Vector3::zero(),
        };

        let mut rng = rand::thread_rng();
        let dir = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        )
        .normalize();

        let mut ray = Ray {
            direction: (normal + dir).normalize(),
            ..*ray
        };
        ray.advance(0.01);

        (mat, ray)
    }
}

pub struct BlackHoleEmitterShader;

impl VolumetricShader for BlackHoleEmitterShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        1.0
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Ray) {
        let mat = MaterialResult {
            albedo: Vector3::zero(),
            emission: Vector3::new(7.0, 3.0, 0.4),
        };

        let mut rng = rand::thread_rng();
        let dir = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        )
        .normalize();

        let ray = Ray {
            direction: dir,
            ..*ray
        };

        (mat, ray)
    }
}

pub struct BlackHoleScatterShader;

impl VolumetricShader for BlackHoleScatterShader {
    fn density_at(&self, _position: Vector3<f64>) -> f64 {
        5.0
    }

    fn material_at(&self, ray: &Ray) -> (MaterialResult, Ray) {
        let mat = MaterialResult {
            albedo: Vector3::new(0.6, 0.6, 0.6),
            emission: Vector3::zero(),
        };

        let mut rng = rand::thread_rng();
        let dir = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        )
        .normalize();

        let ray = Ray {
            direction: dir,
            ..*ray
        };

        (mat, ray)
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
