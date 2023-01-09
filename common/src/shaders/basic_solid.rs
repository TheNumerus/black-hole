use blackhole::material::MaterialResult;
use blackhole::shader::SolidShader;
use blackhole::{Ray, RayKind};

use cgmath::{InnerSpace, Vector3};

use blackhole::math::{rand_unit, rand_unit_vector};

pub struct BasicSolidShader {
    albedo: Vector3<f64>,
    emission: Vector3<f64>,
    metallic: f64,
}

impl BasicSolidShader {
    pub fn new(albedo: Vector3<f64>, emission: Vector3<f64>, metallic: f64) -> Self {
        Self {
            albedo,
            emission,
            metallic,
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
