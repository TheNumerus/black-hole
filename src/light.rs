use cgmath::{InnerSpace, Vector3};

pub struct Light {
    pub location: Vector3<f64>,
    pub color: Vector3<f64>,
    pub strength: f64,
}

impl Light {
    pub fn intensity_at(&self, point: Vector3<f64>) -> Vector3<f64> {
        let dist = (self.location - point).magnitude() - 1.0;
        self.color * (1.0 / dist.powi(2)) * self.strength
    }
}
