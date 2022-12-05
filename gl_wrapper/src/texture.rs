use std::ffi::c_void;
use thiserror::Error;

pub struct Texture2D {
    id: u32,
}

impl Texture2D {
    pub fn new(
        width: u32,
        height: u32,
        data: &[f32],
        format: TextureFormats,
    ) -> Result<Self, TextureError> {
        if (width as usize * height as usize * format.channels() as usize) != data.len() {
            return Err(TextureError::InvalidSrcLength);
        }

        let mut id = 0;

        unsafe {
            gl::GenTextures(1, (&mut id) as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::FLOAT,
                data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Ok(Self { id })
    }

    pub fn update(
        &self,
        width: u32,
        height: u32,
        data: &[f32],
        format: TextureFormats,
    ) -> Result<(), TextureError> {
        if (width as usize * height as usize * format.channels() as usize) != data.len() {
            return Err(TextureError::InvalidSrcLength);
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::FLOAT,
                data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Ok(())
    }

    pub fn bind(&self, unit: u8) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit as u32);
            gl::BindTexture(gl::TEXTURE_2D, self.id)
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, (&self.id) as *const u32);
        }
    }
}

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("Invalid source data length")]
    InvalidSrcLength,
}

pub enum TextureFormats {
    RgbaF32 = gl::RGBA32F as isize,
}

impl TextureFormats {
    pub fn channels(&self) -> u8 {
        match self {
            TextureFormats::RgbaF32 => 4,
        }
    }
}
