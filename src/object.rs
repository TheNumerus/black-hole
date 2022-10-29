use crate::{Ray, Scene};
use cgmath::{Array, ElementWise, InnerSpace, Vector3, Zero};

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
            shading: Shading::Volumetric { density: 1.0 },
        }
    }

    pub fn shade(&self, scene: &Scene, ray: &mut Ray) -> Vector3<f64> {
        match self.shading {
            Shading::Solid => {
                let mut color = Vector3::zero();
                for light in &scene.lights {
                    let light_vec = (light.location - ray.location).normalize();
                    let dot = self.shape.normal(ray.location, 0.0001).dot(light_vec);

                    color += Vector3::from_value(1.0)
                        .mul_element_wise(light.intensity_at(ray.location))
                        * dot.max(0.0);
                }
                color
            }
            Shading::Volumetric { density } => Vector3::new(0.9, 0.9, 0.6),
        }
    }
}

pub enum Shading {
    Solid,
    Volumetric { density: f64 },
}
