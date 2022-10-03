use crate::Ray;
use cgmath::{Angle, Deg, InnerSpace, Matrix4, Transform, Vector3, Zero};

pub struct Camera {
    pub location: Vector3<f32>,
    pub forward: Vector3<f32>,
    pub up: Vector3<f32>,
    pub aspect_ratio: f32,
    pub hor_fov: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            location: Vector3::zero(),
            up: Vector3::new(0.0, 1.0, 0.0),
            forward: Vector3::new(0.0, 0.0, -1.0),
            aspect_ratio: 16.0 / 9.0,
            hor_fov: 90.0,
        }
    }

    pub fn set_forward(&mut self, forward: Vector3<f32>) {
        self.forward = forward.normalize();
    }

    pub fn cast_ray(&self, x: f32, y: f32) -> Ray {
        let side = self.forward.cross(self.up).normalize();
        let up = self.forward.normalize().cross(side).normalize();

        let side = side * (self.hor_fov / 360.0 * std::f32::consts::PI).tan();
        let up = up * (self.hor_fov / 360.0 * std::f32::consts::PI).tan() / self.aspect_ratio;

        let direction = (self.forward + side * (2.0 * x - 1.0) + up * (2.0 * y - 1.0)).normalize();

        Ray {
            location: self.location,
            direction,
        }
    }
}
