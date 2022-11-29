use gl::types::GLuint;
use std::ffi::{c_char, CString};
use std::io::BufRead;
use thiserror::Error;

pub struct ProgramBuilder {
    vert: CString,
    frag: CString,
}

impl ProgramBuilder {
    pub fn new(vert_src: &str, frag_src: &str) -> Self {
        Self {
            vert: CString::new(vert_src).unwrap(),
            frag: CString::new(frag_src).unwrap(),
        }
    }

    pub fn build(self) -> Result<Program, PBError> {
        let mut success: i32 = 0;
        let mut buf = [0_u8; 1024];

        unsafe {
            let vert = gl::CreateShader(gl::VERTEX_SHADER);

            gl::ShaderSource(
                vert,
                1,
                (&self.vert.as_ptr()) as *const *const c_char,
                std::ptr::null(),
            );

            gl::CompileShader(vert);
            gl::GetShaderiv(vert, gl::COMPILE_STATUS, (&mut success) as *mut i32);
            if success != 1 {
                gl::GetShaderInfoLog(
                    vert,
                    1024,
                    std::ptr::null_mut(),
                    (&buf).as_ptr() as *mut c_char,
                );

                let data = if buf.contains(&0) {
                    buf.split(|a| *a == 0).next().unwrap()
                } else {
                    &buf
                };

                return Err(PBError::Compilation(
                    CString::new(data).unwrap().to_string_lossy().to_string(),
                ));
            }

            let frag = gl::CreateShader(gl::FRAGMENT_SHADER);

            gl::ShaderSource(
                frag,
                1,
                (&self.frag.as_ptr()) as *const *const c_char,
                std::ptr::null(),
            );

            gl::CompileShader(frag);
            gl::GetShaderiv(frag, gl::COMPILE_STATUS, (&mut success) as *mut i32);
            if success != 1 {
                buf = [0; 1024];

                gl::GetShaderInfoLog(
                    frag,
                    1024,
                    std::ptr::null_mut(),
                    (&buf).as_ptr() as *mut c_char,
                );

                let data = if buf.contains(&0) {
                    buf.split(|a| *a == 0).next().unwrap()
                } else {
                    &buf
                };

                return Err(PBError::Compilation(
                    CString::new(data).unwrap().to_string_lossy().to_string(),
                ));
            }

            let program = gl::CreateProgram();
            gl::AttachShader(program, vert);
            gl::AttachShader(program, frag);
            gl::LinkProgram(program);

            gl::GetProgramiv(program, gl::LINK_STATUS, (&mut success) as *mut i32);
            if success != 1 {
                buf = [0; 1024];

                gl::GetProgramInfoLog(
                    program,
                    1024,
                    std::ptr::null_mut(),
                    (&buf).as_ptr() as *mut c_char,
                );

                let data = if buf.contains(&0) {
                    buf.split(|a| *a == 0).next().unwrap()
                } else {
                    &buf
                };

                return Err(PBError::Linking(
                    CString::new(data).unwrap().to_string_lossy().to_string(),
                ));
            }

            gl::DeleteShader(vert);
            gl::DeleteShader(frag);

            Ok(Program { id: program })
        }
    }
}

#[derive(Debug, Error)]
pub enum PBError {
    #[error("{0}")]
    Compilation(String),
    #[error("{0}")]
    Linking(String),
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn get_id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) }
    }
}
