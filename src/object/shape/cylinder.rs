use super::Shape;
use crate::object::AABB;
use cgmath::{MetricSpace, Vector3, Zero};

pub struct Cylinder {
    center: Vector3<f64>,
    radius: f64,
    height: f64,
    bounding_box: AABB,
}

impl Cylinder {
    pub fn new() -> Self {
        let mut cylinder = Self {
            center: Vector3::zero(),
            radius: 1.0,
            height: 1.0,
            bounding_box: AABB::new(),
        };

        cylinder.compute_bb();
        cylinder
    }

    pub fn set_radius(&mut self, radius: f64) {
        if radius <= 0.0 {
            panic!("Cylinder radius must be positive number, got {}", radius);
        }

        self.radius = radius;
        self.compute_bb();
    }

    pub fn set_height(&mut self, height: f64) {
        if height <= 0.0 {
            panic!("Cylinder height must be positive number, got {}", height);
        }

        self.height = height;
        self.compute_bb();
    }

    pub fn set_center(&mut self, center: Vector3<f64>) {
        self.center = center;
        self.compute_bb();
    }

    fn compute_bb(&mut self) {
        self.bounding_box = AABB {
            x_min: self.center.x - self.radius - 0.00,
            x_max: self.center.x + self.radius + 0.00,
            y_min: self.center.y - self.height - 0.00,
            y_max: self.center.y + self.height + 0.00,
            z_min: self.center.z - self.radius - 0.00,
            z_max: self.center.z + self.radius + 0.00,
        };
    }
}

impl Shape for Cylinder {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        let relative_point = point - self.center;

        if relative_point.y.abs() >= self.height {
            if relative_point.xz().distance2(self.center.xz()) <= self.radius.powi(2) {
                relative_point.y.abs() - self.height
            } else {
                let dist_to_center = relative_point.xz().distance(self.center.xz()) - self.radius;
                let dist_to_side = relative_point.y.abs() - self.height;

                (dist_to_center * dist_to_center + dist_to_side * dist_to_side).sqrt()
            }
        } else {
            relative_point.xz().distance2(self.center.xz()) - self.radius.powi(2)
        }
    }

    fn bounding_box(&self) -> AABB {
        self.bounding_box
    }
}
