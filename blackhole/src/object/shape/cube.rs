use super::Shape;
use crate::object::AABB;
use cgmath::{Array, Vector3, Zero};

pub struct Cube {
    center: Vector3<f64>,
    scales: Vector3<f64>,
    bounding_box: AABB,
}

impl Cube {
    pub fn new() -> Self {
        let mut cube = Self {
            center: Vector3::zero(),
            scales: Vector3::from_value(1.0),
            bounding_box: AABB::new(),
        };

        cube.compute_bb();
        cube
    }

    pub fn set_center(&mut self, center: Vector3<f64>) {
        self.center = center;
        self.compute_bb();
    }

    pub fn set_scales(&mut self, scales: Vector3<f64>) {
        self.scales = scales;
        self.compute_bb();
    }

    fn compute_bb(&mut self) {
        self.bounding_box = AABB {
            x_min: self.center.x - self.scales.x / 2.0,
            x_max: self.center.x + self.scales.x / 2.0,
            y_min: self.center.y - self.scales.y / 2.0,
            y_max: self.center.y + self.scales.y / 2.0,
            z_min: self.center.z - self.scales.z / 2.0,
            z_max: self.center.z + self.scales.z / 2.0,
        };
    }
}

impl Shape for Cube {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        let mut dist = f64::MIN;

        for i in 0..3 {
            let dist_axis = (self.center[i] - point[i]).abs() - (self.scales[i] / 2.0);

            dist = dist.max(dist_axis);
        }

        dist
    }

    fn bounding_box(&self) -> AABB {
        self.bounding_box
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}
