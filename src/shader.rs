use crate::material::MaterialResult;
use crate::Ray;
use cgmath::Vector3;

pub mod specific;
pub use specific::*;

pub trait SolidShader: Send + Sync {
    fn material_at(&self, ray: &Ray, normal: Vector3<f64>) -> (MaterialResult, Ray);
}

pub trait VolumetricShader: Send + Sync {
    fn density_at(&self, position: Vector3<f64>) -> f64;
    fn material_at(&self, ray: &Ray) -> (MaterialResult, Ray);
}

pub trait BackgroundShader: Send + Sync {
    fn emission_at(&self, direction: Vector3<f64>) -> Vector3<f64>;
}
