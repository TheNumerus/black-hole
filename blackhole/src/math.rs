use crate::GAUSS_LUT;
use cgmath::{InnerSpace, Vector3, VectorSpace};
use rand::Rng;

pub fn rand_unit_vector() -> Vector3<f64> {
    let mut rng = rand::thread_rng();

    let nums = (
        GAUSS_LUT.lookup(rng.gen_range(0.0..1.0)),
        GAUSS_LUT.lookup(rng.gen_range(0.0..1.0)),
        GAUSS_LUT.lookup(rng.gen_range(0.0..1.0)),
    );

    Vector3::new(nums.0, nums.1, nums.2).normalize()
}

pub fn rand_unit() -> f64 {
    let mut rng = rand::thread_rng();

    rng.gen_range(0.0..1.0)
}

pub fn sigmoid(x: f64, slope: f64, center: f64) -> f64 {
    1.0 / (1.0 + std::f64::consts::E.powf(-slope * (x - center)))
}

#[rustfmt::skip]
pub fn blackman_harris(x: f64, n: f64) -> f64 {
    let pi = std::f64::consts::PI;
    
    0.35875
        - 0.48829 * ((2.0 * pi * x) / n).cos()
        + 0.14128 * ((4.0 * pi * x) / n).cos()
        - 0.01168 * ((6.0 * pi * x) / n).cos()
}

pub trait Lerpable: Clone + Copy {
    fn lerp(&self, other: &Self, factor: f64) -> Self;
}

impl Lerpable for f64 {
    fn lerp(&self, other: &Self, factor: f64) -> Self {
        self * (1.0 - factor) + (other * factor)
    }
}

impl Lerpable for Vector3<f64> {
    fn lerp(&self, other: &Self, factor: f64) -> Self {
        <Self as VectorSpace>::lerp(*self, *other, factor)
    }
}
