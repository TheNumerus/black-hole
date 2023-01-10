use blackhole::material::MaterialResult;
use blackhole::shader::{Parameter, Shader, SolidShader};
use blackhole::{Ray, RayKind};

use cgmath::{InnerSpace, Vector3, Zero};

use blackhole::math::{rand_unit, rand_unit_vector};

pub struct BasicSolidShader {
    albedo: Vector3<f64>,
    emission: Vector3<f64>,
    metallic: f64,
}

impl Default for BasicSolidShader {
    fn default() -> Self {
        Self {
            albedo: Vector3::new(0.8, 0.8, 0.8),
            emission: Vector3::zero(),
            metallic: 0.0,
        }
    }
}

impl Shader for BasicSolidShader {
    fn set_parameter(&mut self, name: &str, value: Parameter) {
        match (name, value) {
            ("albedo", Parameter::Vec3(v)) => self.albedo = v,
            ("emission", Parameter::Vec3(e)) => self.emission = e,
            ("metallic", Parameter::Float(m)) => self.metallic = m,
            _ => {}
        }
    }
}

impl SolidShader for BasicSolidShader {
    fn material_at(&self, ray: &Ray, normal: Vector3<f64>) -> (MaterialResult, Option<Ray>) {
        let num = rand_unit();

        let mat = MaterialResult {
            albedo: self.albedo,
            emission: self.emission,
        };

        let mut ray = if num > self.metallic {
            let dir = rand_unit_vector();

            Ray {
                direction: (normal + dir).normalize(),
                kind: RayKind::Secondary,
                ..*ray
            }
        } else {
            let mut ray = ray.reflect(normal);
            ray.kind = RayKind::Secondary;
            ray
        };

        ray.advance(0.01);

        (mat, Some(ray))
    }
}
