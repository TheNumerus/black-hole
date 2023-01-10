use crate::material::MaterialResult;
use crate::Ray;
use cgmath::Vector3;

pub enum Parameter {
    Usize(usize),
    Float(f64),
    Vec3(Vector3<f64>),
}

pub trait Shader: Send + Sync {
    #[allow(unused_variables)]
    /// Method for changing shader parameters. Used in loader.
    fn set_parameter(&mut self, name: &str, value: Parameter) {}
}

pub trait SolidShader: Shader {
    fn material_at(&self, ray: &Ray, normal: Vector3<f64>) -> (MaterialResult, Option<Ray>);
}

pub trait VolumetricShader: Shader {
    fn density_at(&self, position: Vector3<f64>) -> f64;
    fn material_at(&self, ray: &Ray) -> (MaterialResult, Option<Ray>);
}

pub trait BackgroundShader: Shader {
    fn emission_at(&self, ray: &Ray) -> Vector3<f64>;
}
