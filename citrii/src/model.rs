use gl::types::*;

#[derive(Debug)]
pub enum AttributeType {
    Short,
    Float,
}

#[derive(Debug)]
pub struct VaryingAttribute {
    pub dimension: u32,
    pub data_type: AttributeType,
    pub offset: u32,
}

#[derive(Debug)]
pub enum Attribute {
    Varying(VaryingAttribute),
    Short2((i16, i16)),
    Short3((i16, i16, i16)),
}

#[derive(Debug)]
pub struct Model {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    len: usize,
    attribute_map: Vec<(u32, Attribute)>,
}

impl Model {
    pub fn new(vertex: &[u8], index: &[u8], attribute_map: Vec<(u32, Attribute)>, stride: u32) -> Model {
        let mut vao = 0 as GLuint;
        let mut vbo = 0 as GLuint;
        let mut ibo = 0 as GLuint;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            if !vertex.is_empty() {
                gl::GenBuffers(1, &mut vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(gl::ARRAY_BUFFER,
                    vertex.len() as GLsizeiptr,
                    vertex.as_ptr() as *const GLvoid,
                    gl::STATIC_DRAW);
            }

            if !index.is_empty() {
                gl::GenBuffers(1, &mut ibo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                    index.len() as GLsizeiptr,
                    index.as_ptr() as *const GLvoid,
                    gl::STATIC_DRAW);
            }

            for (index, attribute) in &attribute_map {
                match attribute {
                    Attribute::Varying(VaryingAttribute{dimension, data_type, offset}) => {
                        gl::EnableVertexAttribArray(*index as GLuint);
                        gl::VertexAttribPointer(*index as GLuint, *dimension as GLint, match data_type {
                            AttributeType::Short => gl::SHORT,
                            AttributeType::Float => gl::FLOAT,
                        }, gl::FALSE, stride as GLsizei, *offset as *const GLvoid)
                    },
                    _ => {
                        gl::DisableVertexAttribArray(*index as GLuint);
                    }
                }
            }
        }

        Model{vao, vbo, ibo, len: index.len(), attribute_map}
    }

    pub fn draw(&self) {
        unsafe {
            for (index, attribute) in &self.attribute_map {
                match attribute {
                    Attribute::Varying(_) => (),
                    Attribute::Short2((x, y)) =>
                        gl::VertexAttrib2s(*index as GLuint, *x as GLshort, *y as GLshort),
                    Attribute::Short3((x, y, z)) =>
                        gl::VertexAttrib3s(*index as GLuint, *x as GLshort, *y as GLshort, *z as GLshort),

                }
            }
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.len as GLsizei, gl::UNSIGNED_BYTE, 0 as *const GLvoid);
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            if self.vbo != 0 {gl::DeleteBuffers(1, &self.vbo);}
            if self.ibo != 0 {gl::DeleteBuffers(1, &self.ibo);}
        }
    }
}
