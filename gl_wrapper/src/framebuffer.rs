use crate::texture::Texture2D;

pub struct FrameBuffer {
    id: u32,
}

impl FrameBuffer {
    pub fn from_texture(texture: &Texture2D) -> Result<Self, ()> {
        let mut id = 0;

        unsafe {
            gl::GenFramebuffers(1, (&mut id) as *mut u32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.id,
                0,
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(Self { id })
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, (&self.id) as *const u32);
        }
    }
}
