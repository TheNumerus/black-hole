use crate::object::shape::Shape;
use crate::{Ray, Sphere};
use cgmath::{Vector3, Zero};

pub struct Distortion {
    pub strength: f64,
    pub shape: Sphere,
}

impl Distortion {
    pub fn new() -> Self {
        let mut shape = Sphere::new();
        shape.set_radius(2.5);
        shape.set_center(Vector3::zero());

        Self {
            shape,
            strength: 0.4,
        }
    }

    pub fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        self.shape.dist_fn(point)
    }

    pub fn is_inside(&self, point: Vector3<f64>) -> bool {
        self.dist_fn(point) <= 0.0
    }

    pub fn strength(&self, point: Vector3<f64>) -> f64 {
        let x = self.dist_fn(point);
        self.strength / (x + self.shape.radius()).powi(2) * (-x / self.shape.radius())
    }

    pub fn can_ray_hit(&self, ray: &Ray) -> bool {
        self.shape.can_ray_hit(ray)
    }
}
