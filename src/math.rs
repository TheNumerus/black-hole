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

pub fn sigmoid(x: f64, slope: f64, center: f64) -> f64 {
    1.0 / (1.0 + std::f64::consts::E.powf(-slope * (x - center)))
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
