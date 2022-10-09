use crate::{Ray, Scene};
use cgmath::{Array, Vector3};

mod aabb;
mod distortion;
pub mod shape;

pub use aabb::AABB;
pub use distortion::Distortion;
use shape::Shape;

pub struct Object {
    pub shape: Box<dyn Shape>,
    pub shading: Shading,
}

impl Object {
    pub fn solid(shape: Box<dyn Shape>) -> Self {
        Self {
            shape,
            shading: Shading::Solid,
        }
    }

    pub fn volumetric(shape: Box<dyn Shape>) -> Self {
        Self {
            shape,
            shading: Shading::Volumetric,
        }
    }

    pub fn shade(&self, scene: &Scene, ray: &Ray) -> Vector3<f64> {
        match self.shading {
            Shading::Solid => Vector3::from_value(0.5),
            Shading::Volumetric => Vector3::from_value(1.0),
        }
    }
}

pub enum Shading {
    Solid,
    Volumetric,
}
