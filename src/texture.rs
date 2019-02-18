use gl::types::*;

#[derive(Debug)]
pub enum WrapMode {
    Edge,
    Mirror,
    Repeat,
}

impl WrapMode {
    fn to_gl(&self) -> GLenum {
        match self {
            WrapMode::Edge => gl::CLAMP_TO_EDGE,
            WrapMode::Mirror => gl::MIRRORED_REPEAT,
            WrapMode::Repeat => gl::REPEAT
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    handle: GLuint
}

impl Texture {
    pub fn new(width: usize, height: usize, data: &[u8], wrap_u: &WrapMode, wrap_v: &WrapMode) -> Texture {
        let mut handle = 0 as GLuint;
        unsafe {
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_2D, handle);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, width as GLint, height as GLint, 0,
                gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as *const GLvoid);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_u.to_gl() as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_v.to_gl() as GLint);
        }

        Texture{handle}
    }

    pub fn bind(&self, unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
            gl::BindTexture(gl::TEXTURE_2D, self.handle);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}
