#[rustfmt::skip]
pub const QUAD: [f32; 12] = [
    -1.0, -1.0,
    1.0, -1.0,
    -1.0, 1.0,
    -1.0, 1.0,
    1.0, -1.0,
    1.0, 1.0,
];

pub mod framebuffer;
pub mod geometry;
pub mod program;
pub mod renderer;
pub mod texture;
