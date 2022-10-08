use crate::{Ray, Scene, Sphere};
use cgmath::{Array, Vector3, Zero};

pub mod shape;
use shape::Shape;

pub struct Object {
    pub shape: Box<dyn Shape>,
    pub shading: Shading,
}

impl Object {
    pub fn solid(shape: Box<dyn Shape>) -> Self {
        Self {
            shape,
            shading: Shading::Solid,
        }
    }

    pub fn volumetric(shape: Box<dyn Shape>) -> Self {
        Self {
            shape,
            shading: Shading::Volumetric,
        }
    }

    pub fn shade(&self, scene: &Scene, ray: &Ray) -> Vector3<f64> {
        match self.shading {
            Shading::Solid => Vector3::from_value(0.5),
            Shading::Volumetric => Vector3::from_value(1.0),
        }
    }
}

pub enum Shading {
    Solid,
    Volumetric,
}

pub struct AABB {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl AABB {
    pub fn ray_intersect(&self, ray: &Ray) -> bool {
        if ray.location.x < self.x_min && ray.direction.x < 0.0 {
            return false;
        }

        if ray.location.x > self.x_max && ray.direction.x > 0.0 {
            return false;
        }

        if ray.location.y < self.y_min && ray.direction.y < 0.0 {
            return false;
        }

        if ray.location.y > self.y_max && ray.direction.y > 0.0 {
            return false;
        }

        if ray.location.z < self.z_min && ray.direction.z < 0.0 {
            return false;
        }

        if ray.location.z > self.z_max && ray.direction.z > 0.0 {
            return false;
        }

        return true;
    }
}

pub struct Distortion {
    pub strength: f64,
    pub shape: Sphere,
}

impl Distortion {
    pub fn new() -> Self {
        Self {
            shape: Sphere {
                radius: 2.5,
                center: Vector3::zero(),
            },
            strength: 0.4,
        }
    }

    pub fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        self.shape.dist_fn(point)
    }

    pub fn is_inside(&self, point: Vector3<f64>) -> bool {
        self.dist_fn(point) <= 0.0
    }

    pub fn strength(&self, point: Vector3<f64>) -> f64 {
        let x = self.dist_fn(point);
        self.strength / (x + self.shape.radius).powi(2) * (-x / self.shape.radius)
    }

    pub fn can_ray_hit(&self, ray: &Ray) -> bool {
        self.shape.can_ray_hit(ray)
    }
}
