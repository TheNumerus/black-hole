use crate::Ray;
use cgmath::{Deg, InnerSpace, Matrix4, Transform, Vector3, Zero};

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

        let hor_angle = Deg((x - 0.5) * self.hor_fov);
        let ver_angle = Deg(-((y - 0.5) * self.hor_fov) / self.aspect_ratio);

        let up = self.forward.normalize().cross(side).normalize();

        let side_rotation = cgmath::Matrix4::from_axis_angle(up, hor_angle);
        let top_rotation = cgmath::Matrix4::from_axis_angle(side.normalize(), ver_angle);

        let mut direction = side_rotation.transform_vector(self.forward);
        direction = top_rotation.transform_vector(direction);

        Ray {
            location: self.location + direction * rand::random() * 0.1,
            direction,
        }
    }
}
