use crate::{Ray, RayKind};
use cgmath::{Deg, InnerSpace, Matrix3, SquareMatrix, Vector3, Zero};

#[derive(Clone)]
pub struct Camera {
    pub location: Vector3<f64>,
    pub hor_fov: f64,
    rot_mat: Matrix3<f64>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            location: Vector3::zero(),
            hor_fov: 90.0,
            rot_mat: Matrix3::identity(),
        }
    }

    pub fn set_rotation(&mut self, rotation: Vector3<f64>) {
        self.rot_mat = Matrix3::from_angle_y(Deg(rotation.y))
            * Matrix3::from_angle_x(Deg(rotation.x))
            * Matrix3::from_angle_z(Deg(rotation.z));
    }

    pub fn side(&self) -> Vector3<f64> {
        self.rot_mat * Vector3::new(1.0, 0.0, 0.0)
    }

    pub fn up(&self) -> Vector3<f64> {
        self.rot_mat * Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn cast_ray(&self, x: f64, y: f64, aspect_ratio: f64) -> Ray {
        let side = self.rot_mat * Vector3::new(1.0, 0.0, 0.0);
        let up = self.rot_mat * Vector3::new(0.0, 1.0, 0.0);
        let forward = self.rot_mat * Vector3::new(0.0, 0.0, -1.0);

        let side = side * (self.hor_fov / 360.0 * std::f64::consts::PI).tan();
        let up = up * (self.hor_fov / 360.0 * std::f64::consts::PI).tan() / aspect_ratio;

        let direction = (forward + side * (2.0 * x - 1.0) - up * (2.0 * y - 1.0)).normalize();

        Ray {
            location: self.location,
            direction,
            steps_taken: 0,
            kind: RayKind::Primary,
        }
    }

    pub fn cast_ray_panoramic(&self, x: f64, y: f64) -> Ray {
        let angle_y = (1.0 - y) * 2.0 - 1.0;

        let angle_x = (x * 2.0 * std::f64::consts::PI).cos() * (1.0 - angle_y.abs().powi(2));
        let angle_z = (x * 2.0 * std::f64::consts::PI).sin() * (1.0 - angle_y.abs().powi(2));

        let direction = Vector3::new(angle_x, angle_y, angle_z).normalize();

        Ray {
            location: self.location,
            direction,
            steps_taken: 0,
            kind: RayKind::Primary,
        }
    }
}
