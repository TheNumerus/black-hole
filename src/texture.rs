use crate::math::rand_unit_vector;
use cgmath::{InnerSpace, Vector3, Zero};
use rand::Rng;

pub trait Texture3D: Send + Sync {
    type Output;

    fn color_at(&self, position: Vector3<f64>) -> Self::Output;
}

#[derive(Clone)]
pub struct NoiseTexture3D {
    scale: f64,
    randoms: Vec<Vector3<f64>>,
    permutations: [Vec<usize>; 3],
}

impl NoiseTexture3D {
    pub fn new(scale: f64) -> Self {
        let mut randoms = Vec::with_capacity(256);
        let mut permutations = [
            Vec::with_capacity(256),
            Vec::with_capacity(256),
            Vec::with_capacity(256),
        ];
        let mut rng = rand::thread_rng();

        for i in 0..256 {
            randoms.push(rand_unit_vector());
            permutations[0].push(i);
            permutations[1].push(i);
            permutations[2].push(i);
        }

        for i in 0..3 {
            for x in 0..256 {
                let target = rng.gen_range(0..256);

                permutations[i].swap(x, target);
            }
        }

        Self {
            scale,
            randoms,
            permutations,
        }
    }

    fn sample(&self, position: Vector3<f64>) -> f64 {
        let position = position * self.scale;

        let mut u = position.x - position.x.floor();
        let mut v = position.y - position.y.floor();
        let mut w = position.z - position.z.floor();

        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let x = (position.x).floor() as isize;
        let y = (position.y).floor() as isize;
        let z = (position.z).floor() as isize;

        let mut values = [[[Vector3::zero(); 2]; 2]; 2];

        for dx in 0..2 {
            for dy in 0..2 {
                for dz in 0..2 {
                    values[dx][dy][dz] = self.randoms[self.permutations[0]
                        [((x + dx as isize) & 255) as usize]
                        ^ self.permutations[1][((y + dy as isize) & 255) as usize]
                        ^ self.permutations[2][((z + dz as isize) & 255) as usize]];
                }
            }
        }

        Self::perlin_filter(values, u, v, w) * 0.5 + 0.5
    }

    fn perlin_filter(inputs: [[[Vector3<f64>; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut acc = 0.0;
        for x in 0..2 {
            let xf = x as f64;
            for y in 0..2 {
                let yf = y as f64;
                for z in 0..2 {
                    let zf = z as f64;
                    let weight = Vector3::new(u - xf, v - yf, w - zf);
                    acc += (xf * uu + (1.0 - xf) * (1.0 - uu))
                        * (yf * vv + (1.0 - yf) * (1.0 - vv))
                        * (zf * ww + (1.0 - zf) * (1.0 - ww))
                        * inputs[x][y][z].dot(weight);
                }
            }
        }

        acc
    }
}

impl Texture3D for NoiseTexture3D {
    type Output = f64;

    fn color_at(&self, position: Vector3<f64>) -> Self::Output {
        self.sample(position)
    }
}
