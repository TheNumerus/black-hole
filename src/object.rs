use crate::Ray;
use cgmath::{MetricSpace, Vector3, Zero};

pub struct Sphere {
    pub center: Vector3<f32>,
    pub radius: f32,
    color: Vector3<f32>,
}

impl Sphere {
    pub fn new() -> Self {
        Self {
            center: Vector3::zero(),
            radius: 1.0,
            color: Vector3::new(0.5, 0.5, 0.5),
        }
    }
}

impl Renderable for Sphere {
    fn dist_fn(&self, point: Vector3<f32>) -> f32 {
        ((point).distance(self.center) - self.radius)
    }

    fn color(&self, point: Vector3<f32>) -> Vector3<f32> {
        let latitude = (((point - self.center) / self.radius).y.acos() * 9.0).floor() as i32;

        if latitude % 2 == 0 {
            Vector3::new(0.2, 0.2, 0.2)
        } else {
            Vector3::new(0.8, 0.8, 0.8)
        }
    }
}

pub trait Renderable {
    fn dist_fn(&self, point: Vector3<f32>) -> f32;
    fn color(&self, point: Vector3<f32>) -> Vector3<f32>;
    fn can_ray_hit(&self, ray: &Ray) -> bool {
        true
    }
}
