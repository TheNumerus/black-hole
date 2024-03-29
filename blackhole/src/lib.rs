use cgmath::{InnerSpace, Vector3};

use once_cell::sync::Lazy;

pub mod camera;
pub mod filter;
pub mod frame;
pub mod framebuffer;
pub mod lut;
pub mod marcher;
pub mod material;
pub mod math;
pub mod object;
pub mod scene;
pub mod shader;
pub mod texture;

use crate::lut::LookupTable;

pub static GAUSS_LUT: Lazy<LookupTable<f64>> = Lazy::new(gen_gauss_dist);
pub static BLACKBODY_LUT: Lazy<LookupTable<Vector3<f64>>> = Lazy::new(gen_bb_dist);

#[derive(Debug, Copy, Clone)]
pub enum RayKind {
    Primary,
    Secondary,
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub location: Vector3<f64>,
    pub direction: Vector3<f64>,
    pub steps_taken: usize,
    pub kind: RayKind,
}

impl Ray {
    pub fn advance(&mut self, dist: f64) {
        self.location += self.direction * dist;
        self.steps_taken += 1;
    }

    pub fn reflect(&self, normal: Vector3<f64>) -> Self {
        Ray {
            location: self.location,
            direction: self.direction - 2.0 * self.direction.dot(normal) * normal,
            steps_taken: 0,
            kind: RayKind::Secondary,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RenderMode {
    Samples,
    Normal,
    Shaded,
}

fn gen_gauss_dist() -> LookupTable<f64> {
    let mut data = Vec::new();

    let mut integral = 0.0;

    let base = 1.0 / (2.0 * std::f64::consts::PI).sqrt();

    let mut last_integral = 0.0;

    for i in -500..=500 {
        let f = i as f64 / 100.0;

        let slice = std::f64::consts::E.powf(-f.powi(2) / 2.0);

        integral += 0.01 * slice + ((last_integral - slice) / 2.0) * 0.01;

        last_integral = slice;

        let item = (base * integral, f);

        data.push(item);
    }

    LookupTable::from_vec(data)
}

fn gen_bb_dist() -> LookupTable<Vector3<f64>> {
    LookupTable::from_vec(vec![
        (500.0, Vector3::new(0.0, 0.0, 0.0)),
        (1000.0, Vector3::new(1.0, 0.0, 0.0)),
        (2000.0, Vector3::new(1.0, 0.2, 0.0)),
        (3000.0, Vector3::new(1.0, 0.8, 0.2)),
        (6500.0, Vector3::new(1.0, 1.0, 1.0)),
    ])
}
