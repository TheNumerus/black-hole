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

    fn can_ray_hit(&self, ray: &Ray) -> bool {
        let l = self.center - ray.location;
        let tca = l.dot(ray.direction);
        let d2 = l.dot(l) - tca * tca;
        if d2 > self.radius.powi(2) {
            return false;
        }

        true
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

    fn color(&self, _point: Vector3<f64>) -> Vector3<f64> {
        self.color
    }

    fn bounding_box(&self) -> Option<[f64; 6]> {
        Some([
            self.center.x - self.radius - 0.02,
            self.center.x + self.radius + 0.02,
            self.center.y - self.height - 0.02,
            self.center.y + self.height + 0.02,
            self.center.z - self.radius - 0.02,
            self.center.z + self.radius + 0.02,
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
            radius: 2.5,
            strength: 0.4,
        }
    }

    pub fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        (point - self.center).magnitude() - self.radius
    }

    pub fn is_inside(&self, point: Vector3<f64>) -> bool {
        self.dist_fn(point) <= 0.0
    }

    pub fn strength(&self, point: Vector3<f64>) -> f64 {
        let x = self.dist_fn(point);
        self.strength / (x + self.radius).powi(2) * (-x / self.radius)
    }

    pub fn can_ray_hit(&self, ray: &Ray) -> bool {
        let l = self.center - ray.location;
        let tca = l.dot(ray.direction);
        let d2 = l.dot(l) - tca * tca;
        if d2 > self.radius.powi(2) {
            return false;
        }
        return true;
    }
}

pub enum Composite {
    Diff(Box<dyn Renderable>, Box<dyn Renderable>),
}

impl Renderable for Composite {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        match self {
            Composite::Diff(a, b) => {
                let a = a.dist_fn(point.clone());
                let b = b.dist_fn(point);

                (a).max(-b)
            }
        }
    }

    fn color(&self, point: Vector3<f64>) -> Vector3<f64> {
        match self {
            Composite::Diff(a, _) => a.color(point),
        }
    }

    fn bounding_box(&self) -> Option<[f64; 6]> {
        None
    }
}
