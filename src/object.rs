use crate::Ray;
use cgmath::{InnerSpace, MetricSpace, Vector3, Zero};

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    color: Vector3<f64>,
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
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        (point - self.center).magnitude() - self.radius
    }

    fn color(&self, point: Vector3<f64>) -> Vector3<f64> {
        let latitude = (((point - self.center) / self.radius).y.acos() * 9.0).floor() as i32;

        if latitude % 2 == 0 {
            Vector3::new(0.2, 0.2, 0.2)
        } else {
            Vector3::new(0.8, 0.8, 0.8)
        }
    }

    fn bounding_box(&self) -> Option<[f64; 6]> {
        Some([
            self.center.x - self.radius,
            self.center.x + self.radius,
            self.center.y - self.radius,
            self.center.y + self.radius,
            self.center.z - self.radius,
            self.center.z + self.radius,
        ])
    }
}

pub struct Cylinder {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub height: f64,
    color: Vector3<f64>,
}

impl Cylinder {
    pub fn new() -> Self {
        Self {
            center: Vector3::zero(),
            radius: 1.0,
            height: 1.0,
            color: Vector3::new(0.5, 0.5, 0.5),
        }
    }
}

impl Renderable for Cylinder {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        let relative_point = point - self.center;

        let dst =
            (relative_point.x * relative_point.x + relative_point.z * relative_point.z).sqrt();

        let phase = {
            if relative_point.z >= 0.0 {
                (relative_point.x / dst).acos()
            } else {
                -(relative_point.x / dst).acos()
            }
        };
        let displacement = (phase * 16.0 + dst * 16.0).sin() * 0.01 + 0.01;

        if relative_point.y.abs() >= self.height {
            if relative_point.xz().distance(self.center.xz()) <= self.radius {
                relative_point.y.abs() - self.height - displacement
            } else {
                let dist_to_center = relative_point.xz().distance(self.center.xz()) - self.radius;
                let dist_to_side = relative_point.y.abs() - self.height;

                (dist_to_center * dist_to_center + dist_to_side * dist_to_side).sqrt()
                    - displacement
            }
        } else {
            relative_point.xz().distance(self.center.xz()) - self.radius - displacement
        }
    }

    fn color(&self, _point: Vector3<f64>) -> Vector3<f64> {
        self.color
    }

    fn bounding_box(&self) -> Option<[f64; 6]> {
        Some([
            self.center.x - self.radius - 0.1,
            self.center.x + self.radius + 0.1,
            self.center.y - self.height - 0.1,
            self.center.y + self.height + 0.1,
            self.center.z - self.radius - 0.1,
            self.center.z + self.radius + 0.1,
        ])
    }
}

pub trait Renderable {
    fn dist_fn(&self, point: Vector3<f64>) -> f64;
    fn color(&self, point: Vector3<f64>) -> Vector3<f64>;
    fn bounding_box(&self) -> Option<[f64; 6]>;

    fn can_ray_hit(&self, ray: &Ray) -> bool {
        match self.bounding_box() {
            None => true,
            Some([x_min, x_max, y_min, y_max, z_min, z_max]) => {
                if ray.location.x < x_min && ray.direction.x < 0.0 {
                    return false;
                }

                if ray.location.x > x_max && ray.direction.x > 0.0 {
                    return false;
                }

                if ray.location.y < y_min && ray.direction.y < 0.0 {
                    return false;
                }

                if ray.location.y > y_max && ray.direction.y > 0.0 {
                    return false;
                }

                if ray.location.z < z_min && ray.direction.z < 0.0 {
                    return false;
                }

                if ray.location.z > z_max && ray.direction.z > 0.0 {
                    return false;
                }

                true
            }
        }
    }
}

pub struct Distortion {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub strength: f64,
}

impl Distortion {
    pub fn new() -> Self {
        Self {
            center: Vector3::zero(),
            radius: 2.0,
            strength: 3.0,
        }
    }

    pub fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        (point - self.center).magnitude() - self.radius
    }

    pub fn is_inside(&self, point: Vector3<f64>) -> bool {
        (point - self.center).magnitude() - self.radius <= 0.0
    }

    pub fn strength(&self, point: Vector3<f64>) -> f64 {
        self.strength * (((point - self.center).magnitude() - self.radius) / self.radius).powf(2.0)
    }
}
