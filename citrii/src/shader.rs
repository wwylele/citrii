use cgmath::prelude::*;
use gl::types::*;

unsafe fn create_shader(code: &str, shader_type: GLenum) -> GLuint {
    let handle = gl::CreateShader(shader_type);
    let code_str = code.as_ptr() as *const GLchar;
    let code_len = code.len() as GLint;
    gl::ShaderSource(handle, 1, &code_str, &code_len);
    gl::CompileShader(handle);
    let mut status = 0 as GLint;
    gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);
    if status != gl::TRUE as GLint {
        let mut buf = [0u8; 512];
        let mut buf_len = 0 as GLint;
        gl::GetShaderInfoLog(handle, 512, &mut buf_len, buf.as_mut_ptr() as *mut GLchar);
        panic!(
            "Shader error: {}",
            std::str::from_utf8(&buf[0..buf_len as usize]).expect("Shader errorception")
        );
    }
    handle
}

pub struct Shader {
    handle: GLuint,
    uniform_map: std::cell::RefCell<std::collections::HashMap<String, GLint>>,
}

impl Shader {
    pub fn new(vertex_shader: &str, fragment_shader: &str) -> Shader {
        unsafe {
            let handle = gl::CreateProgram();
            let vs = create_shader(vertex_shader, gl::VERTEX_SHADER);
            let fs = create_shader(fragment_shader, gl::FRAGMENT_SHADER);
            gl::AttachShader(handle, vs);
            gl::AttachShader(handle, fs);
            gl::LinkProgram(handle);
            let mut status = 0 as GLint;
            gl::GetProgramiv(handle, gl::LINK_STATUS, &mut status);
            if status != gl::TRUE as GLint {
                let mut buf = [0u8; 512];
                let mut buf_len = 0 as GLint;
                gl::GetProgramInfoLog(handle, 512, &mut buf_len, buf.as_mut_ptr() as *mut GLchar);
                panic!(
                    "Shader link error: {}",
                    std::str::from_utf8(&buf[0..buf_len as usize]).expect("Shader errorspection")
                );
            }
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
            Shader {
                handle,
                uniform_map: std::cell::RefCell::new(std::collections::HashMap::new()),
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.handle);
        }
    }

    pub fn set_uniform_vec<T: cgmath::Array<Element = f32>>(&self, name: &str, v: &T) {
        unsafe {
            match T::len() {
                2 => gl::Uniform2fv(self.uniform_location(name), 1, v.as_ptr()),
                3 => gl::Uniform3fv(self.uniform_location(name), 1, v.as_ptr()),
                4 => gl::Uniform4fv(self.uniform_location(name), 1, v.as_ptr()),
                _ => panic!("Wrong vector size"),
            }
        }
    }

    pub fn set_uniform_i(&self, name: &str, v: i32) {
        unsafe {
            gl::Uniform1i(self.uniform_location(name), v);
        }
    }

    pub fn set_uniform_mat4(&self, name: &str, v: &cgmath::Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniform_location(name), 1, gl::FALSE, v.as_ptr());
        }
    }

    pub fn set_uniform_mat3(&self, name: &str, v: &cgmath::Matrix3<f32>) {
        unsafe {
            gl::UniformMatrix3fv(self.uniform_location(name), 1, gl::FALSE, v.as_ptr());
        }
    }

    fn uniform_location(&self, name: &str) -> GLint {
        let mut uniform_map = self.uniform_map.borrow_mut();
        let cached = uniform_map.get(name);
        cached.cloned().unwrap_or_else(|| unsafe {
            let c_string = std::ffi::CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.handle, c_string.as_ptr());
            uniform_map.insert(String::from(name), location);
            location
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.handle);
        }
    }
}
