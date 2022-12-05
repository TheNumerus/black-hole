use crate::geometry::Geometry;
use crate::program::Program;

pub struct GlRenderer {
    current_program: u32,
}

impl GlRenderer {
    pub fn new() -> Self {
        Self { current_program: 0 }
    }

    pub fn draw(&mut self, geometry: &Geometry, program: &Program) {
        let p_id = program.get_id();
        if self.current_program != p_id {
            unsafe { gl::UseProgram(p_id) }
            self.current_program = p_id;
        }

        unsafe {
            gl::BindVertexArray(geometry.vao());
            gl::DrawArrays(gl::TRIANGLES, 0, geometry.vertices() as i32);
        }
    }

    pub fn resize(&self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }
}
