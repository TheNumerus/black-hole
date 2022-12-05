use crate::Ray;
use std::sync::Arc;

mod aabb;
mod distortion;
pub mod shape;

use crate::material::MaterialResult;
use crate::shader::{SolidShader, VolumetricShader};

pub use aabb::AABB;
pub use distortion::Distortion;
use shape::Shape;

pub struct Object {
    pub shape: Box<dyn Shape>,
    pub shading: Shading,
}

impl Object {
    pub fn solid(shape: Box<dyn Shape>, shader: Arc<dyn SolidShader>) -> Self {
        Self {
            shape,
            shading: Shading::Solid(shader),
        }
    }

    pub fn volumetric(shape: Box<dyn Shape>, shader: Arc<dyn VolumetricShader>) -> Self {
        Self {
            shape,
            shading: Shading::Volumetric(shader),
        }
    }

    pub fn shade(&self, ray: &Ray) -> (MaterialResult, Option<Ray>) {
        match &self.shading {
            Shading::Solid(s) => {
                let eps = 0.00001;
                let normal = self.shape.normal(ray.location, eps);

                s.material_at(ray, normal)
            }
            Shading::Volumetric(v) => v.material_at(ray),
        }
    }
}

pub enum Shading {
    Solid(Arc<dyn SolidShader>),
    Volumetric(Arc<dyn VolumetricShader>),
}
