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

    fn bounding_box(&self) -> Option<[f32; 6]> {
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
    pub center: Vector3<f32>,
    pub radius: f32,
    pub height: f32,
    color: Vector3<f32>,
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
    fn dist_fn(&self, point: Vector3<f32>) -> f32 {
        let relative_point = point - self.center;

        if relative_point.y.abs() >= self.height {
            if relative_point.xz().distance(self.center.xz()) <= self.radius {
                relative_point.y.abs() - self.height
            } else {
                let dist_to_center = relative_point.xz().distance(self.center.xz()) - self.radius;
                let dist_to_side = relative_point.y - self.height;
                let sign = dist_to_center > 0.0;

                let res = (dist_to_center * dist_to_center + dist_to_side * dist_to_side).sqrt();

                if sign {
                    res
                } else {
                    -res
                }
            }
        } else {
            relative_point.xz().distance(self.center.xz()) - self.radius
        }
    }

    fn color(&self, point: Vector3<f32>) -> Vector3<f32> {
        self.color
    }

    fn bounding_box(&self) -> Option<[f32; 6]> {
        Some([
            self.center.x - self.radius,
            self.center.x + self.radius,
            self.center.y - self.height,
            self.center.y + self.height,
            self.center.z - self.radius,
            self.center.z + self.radius,
        ])
    }
}

pub trait Renderable {
    fn dist_fn(&self, point: Vector3<f32>) -> f32;
    fn color(&self, point: Vector3<f32>) -> Vector3<f32>;
    fn bounding_box(&self) -> Option<[f32; 6]>;

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
