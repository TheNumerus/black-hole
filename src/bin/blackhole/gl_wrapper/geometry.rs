use std::ffi::c_void;
use thiserror::Error;

pub struct GeometryBuilder<'a> {
    attributes: Vec<VertexAttribute>,
    data: &'a [f32],
}

impl<'a> GeometryBuilder<'a> {
    pub fn new(data: &'a [f32]) -> Self {
        Self {
            data,
            attributes: Vec::new(),
        }
    }

    pub fn with_attribute(mut self, attr: VertexAttribute) -> Self {
        self.attributes.push(attr);
        self
    }

    pub fn build(self) -> Result<Geometry, GBError> {
        let total_len: usize = self.attributes.iter().map(|a| a.size()).sum();

        if self.data.len() % total_len != 0 {
            return Err(GBError::InvalidDataLength);
        }

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, (&mut vao) as *mut u32);
            gl::GenBuffers(1, (&mut vbo) as *mut u32);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.data.len() * std::mem::size_of::<f32>()) as isize,
                self.data.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            let mut offset = 0;

            for (i, attr) in self.attributes.iter().enumerate() {
                gl::VertexAttribPointer(
                    i as u32,
                    attr.size() as i32,
                    gl::FLOAT,
                    gl::FALSE,
                    (total_len * std::mem::size_of::<f32>()) as i32,
                    offset as *const c_void,
                );
                offset += attr.size();
                gl::EnableVertexAttribArray(i as u32);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        let vertices = self.data.len() / total_len;

        Ok(Geometry { vao, vbo, vertices })
    }
}

#[derive(Debug, Error)]
pub enum GBError {
    #[error("Invalid data length for given attributes")]
    InvalidDataLength,
}

pub enum VertexAttribute {
    Float,
    Vec2,
    Vec3,
}

impl VertexAttribute {
    pub fn size(&self) -> usize {
        match self {
            VertexAttribute::Float => 1,
            VertexAttribute::Vec2 => 2,
            VertexAttribute::Vec3 => 3,
        }
    }
}

pub struct Geometry {
    vao: u32,
    vbo: u32,
    vertices: usize,
}

impl Geometry {
    pub fn vao(&self) -> u32 {
        self.vao
    }
    pub fn vertices(&self) -> usize {
        self.vertices
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, (&self.vbo) as *const u32);
            gl::DeleteVertexArrays(1, (&self.vao) as *const u32);
        }
    }
}
