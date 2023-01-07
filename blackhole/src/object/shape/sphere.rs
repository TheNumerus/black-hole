use super::Shape;
use crate::object::AABB;
use crate::Ray;
use cgmath::{InnerSpace, Vector3, Zero};

#[derive(Clone)]
pub struct Sphere {
    center: Vector3<f64>,
    radius: f64,
    bounding_box: AABB,
}

impl Sphere {
    pub fn new() -> Self {
        let mut sphere = Self {
            center: Vector3::zero(),
            radius: 1.0,
            bounding_box: AABB::new(),
        };

        sphere.compute_bb();
        sphere
    }

    pub fn set_center(&mut self, center: Vector3<f64>) {
        self.center = center;
        self.compute_bb();
    }

    pub fn set_radius(&mut self, radius: f64) {
        if radius <= 0.0 {
            panic!("Sphere radius must be positive number, got {}", radius);
        }

        self.radius = radius;
        self.compute_bb();
    }

    pub fn center(&self) -> Vector3<f64> {
        self.center
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    fn compute_bb(&mut self) {
        self.bounding_box = AABB {
            x_min: self.center.x - self.radius,
            x_max: self.center.x + self.radius,
            y_min: self.center.y - self.radius,
            y_max: self.center.y + self.radius,
            z_min: self.center.z - self.radius,
            z_max: self.center.z + self.radius,
        };
    }
}

impl Shape for Sphere {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        (point - self.center).magnitude() - self.radius
    }

    fn bounding_box(&self) -> AABB {
        self.bounding_box
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

    fn normal(&self, position: Vector3<f64>, _epsilon: f64) -> Vector3<f64> {
        (position - self.center).normalize()
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}
