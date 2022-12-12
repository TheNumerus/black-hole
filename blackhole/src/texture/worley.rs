use super::Texture3D;
use crate::math::rand_unit_vector;
use cgmath::{Array, ElementWise, MetricSpace, Vector3};

pub struct WorleyTexture3D {
    scale: f64,
    randoms: Vec<Vector3<f64>>,
}

impl WorleyTexture3D {
    pub fn new(scale: f64) -> Self {
        let mut randoms = Vec::new();

        for _ in 0..256 {
            randoms.push((rand_unit_vector() * 0.5).add_element_wise(Vector3::from_value(0.5)))
        }

        Self { scale, randoms }
    }

    fn sample(&self, position: Vector3<f64>) -> f64 {
        let position = self.scale * position;

        let mut dist = 3.0;

        for x in 0..3 {
            let dx = x - 1;
            for y in 0..3 {
                let dy = y - 1;
                for z in 0..3 {
                    let dz = z - 1;
                    let cell = ((position.x.floor() as i32 + dx) & 255) as usize
                        ^ ((position.y.floor() as i32 + dy) & 255) as usize
                        ^ ((position.z.floor() as i32 + dz) & 255) as usize;

                    let point = self.randoms[cell]
                        + Vector3::new(
                            position.x.floor() + dx as f64,
                            position.y.floor() + dy as f64,
                            position.z.floor() + dz as f64,
                        );

                    let d = point.distance(position);

                    if d < dist {
                        dist = d;
                    }
                }
            }
        }

        dist
    }
}

impl Texture3D for WorleyTexture3D {
    type Output = f64;

    fn color_at(&self, position: Vector3<f64>) -> Self::Output {
        self.sample(position)
    }
}
