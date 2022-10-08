use super::Shape;
use crate::object::AABB;
use cgmath::{MetricSpace, Vector3, Zero};

pub struct Cylinder {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub height: f64,
}

impl Cylinder {
    pub fn new() -> Self {
        Self {
            center: Vector3::zero(),
            radius: 1.0,
            height: 1.0,
        }
    }
}

impl Shape for Cylinder {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        let relative_point = point - self.center;

        if relative_point.y.abs() >= self.height {
            if relative_point.xz().distance(self.center.xz()) <= self.radius {
                relative_point.y.abs() - self.height
            } else {
                let dist_to_center = relative_point.xz().distance(self.center.xz()) - self.radius;
                let dist_to_side = relative_point.y.abs() - self.height;

                (dist_to_center * dist_to_center + dist_to_side * dist_to_side).sqrt()
            }
        } else {
            relative_point.xz().distance(self.center.xz()) - self.radius
        }
    }

    fn bounding_box(&self) -> AABB {
        AABB {
            x_min: self.center.x - self.radius - 0.02,
            x_max: self.center.x + self.radius + 0.02,
            y_min: self.center.y - self.height - 0.02,
            y_max: self.center.y + self.height + 0.02,
            z_min: self.center.z - self.radius - 0.02,
            z_max: self.center.z + self.radius + 0.02,
        }
    }
}