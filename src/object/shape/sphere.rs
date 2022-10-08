use super::Shape;
use crate::object::AABB;
use crate::Ray;
use cgmath::{InnerSpace, Vector3, Zero};

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
}

impl Sphere {
    pub fn new() -> Self {
        Self {
            center: Vector3::zero(),
            radius: 1.0,
        }
    }
}

impl Shape for Sphere {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        (point - self.center).magnitude() - self.radius
    }

    fn bounding_box(&self) -> AABB {
        AABB {
            x_min: self.center.x - self.radius,
            x_max: self.center.x + self.radius,
            y_min: self.center.y - self.radius,
            y_max: self.center.y + self.radius,
            z_min: self.center.z - self.radius,
            z_max: self.center.z + self.radius,
        }
    }

    fn can_ray_hit(&self, ray: &Ray) -> bool {
        let l = self.center - ray.location;
        let tca = l.dot(ray.direction);
        let d2 = l.dot(l) - tca * tca;
        if d2 > self.radius.powi(2) {
            return false;
        }

        true
    }
}
