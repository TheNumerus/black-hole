use crate::object::shape::{Shape, Sphere};
use crate::Ray;
use cgmath::{Vector3, Zero};

#[derive(Clone)]
pub struct Distortion {
    pub strength: f64,
    pub shape: Sphere,
}

impl Distortion {
    pub fn new() -> Self {
        let mut shape = Sphere::new();
        shape.set_radius(5.0);
        shape.set_center(Vector3::zero());

        Self {
            shape,
            strength: 0.3,
        }
    }

    pub fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        self.shape.dist_fn(point)
    }

    pub fn strength(&self, point: Vector3<f64>) -> f64 {
        let x = self.dist_fn(point) + self.shape.radius();
        self.strength / (x).powi(2)
    }

    pub fn can_ray_hit(&self, ray: &Ray) -> bool {
        self.shape.can_ray_hit(ray)
    }
}
