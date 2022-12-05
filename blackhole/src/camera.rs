use crate::{Ray, RayKind};
use cgmath::{InnerSpace, Vector3, Zero};

pub struct Camera {
    pub location: Vector3<f64>,
    forward: Vector3<f64>,
    up: Vector3<f64>,
    pub hor_fov: f64,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            location: Vector3::zero(),
            up: Vector3::new(0.0, 1.0, 0.0),
            forward: Vector3::new(0.0, 0.0, -1.0),
            hor_fov: 90.0,
        }
    }

    pub fn set_forward(&mut self, forward: Vector3<f64>) {
        self.forward = forward.normalize();
    }

    pub fn set_up(&mut self, up: Vector3<f64>) {
        self.up = up.normalize();
    }

    pub fn cast_ray(&self, x: f64, y: f64, aspect_ratio: f64) -> Ray {
        let side = self.forward.cross(self.up);
        let up = self.forward.cross(side);

        let side = side * (self.hor_fov / 360.0 * std::f64::consts::PI).tan();
        let up = up * (self.hor_fov / 360.0 * std::f64::consts::PI).tan() / aspect_ratio;

        let direction = (self.forward + side * (2.0 * x - 1.0) + up * (2.0 * y - 1.0)).normalize();

        Ray {
            location: self.location,
            direction,
            steps_taken: 0,
            kind: RayKind::Primary,
        }
    }
}
