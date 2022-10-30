use cgmath::{Vector3, Zero};

pub struct MaterialResult {
    pub emission: Vector3<f64>,
    pub albedo: Vector3<f64>,
}

impl MaterialResult {
    pub fn black() -> Self {
        Self {
            emission: Vector3::zero(),
            albedo: Vector3::zero(),
        }
    }
}
