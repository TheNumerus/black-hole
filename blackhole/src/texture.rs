use cgmath::Vector3;

mod perlin;
mod worley;

pub use perlin::NoiseTexture3D;
pub use worley::WorleyTexture3D;

pub trait Texture3D: Send + Sync {
    type Output;

    fn color_at(&self, position: Vector3<f64>) -> Self::Output;
}
