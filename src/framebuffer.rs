use cgmath::Vector3;
use std::ops::{Add, AddAssign, Mul};

pub struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Pixel>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![Pixel::black(); width * height],
        }
    }

    pub fn buffer_mut(&mut self) -> &mut Vec<Pixel> {
        &mut self.buffer
    }

    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut Pixel> {
        let index = x + y * self.width;

        if index >= self.width * self.height {
            return None;
        }

        Some(&mut self.buffer[index])
    }

    /// Hope that pixel is basically \[f32;4]
    pub unsafe fn as_f32_slice(&self) -> &[f32] {
        let size = self.width * self.height * 4;

        std::slice::from_raw_parts(self.buffer.as_ptr() as *const f32, size)
    }
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Pixel {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl Add<Pixel> for Pixel {
    type Output = Pixel;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl AddAssign for Pixel {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += 0.0;
    }
}

impl Mul<f32> for Pixel {
    type Output = Pixel;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl From<Vector3<f64>> for Pixel {
    fn from(v: Vector3<f64>) -> Self {
        Self::new(v.x as f32, v.y as f32, v.z as f32, 1.0)
    }
}
