use crate::model;
use crate::shader;
use crate::texture;

pub struct RectRenderer {
    rect_shader: shader::Shader,
    rect: model::Model,
}

pub enum Filling<'a> {
    Color(f32, f32, f32, f32),
    Texture(
        &'a texture::Texture,
        ((f32, f32), (f32, f32)),
        (f32, f32, f32),
    ),
}

impl RectRenderer {
    pub fn new() -> RectRenderer {
        let rect_shader =
            shader::Shader::new(include_str!("rect.v.glsl"), include_str!("rect.f.glsl"));
        let rect = model::Model::new(&[], &[0, 1, 2, 1, 3, 2], vec![], 0);
        RectRenderer { rect_shader, rect }
    }

    pub fn render(&self, ((x0, y0), (x1, y1)): ((f32, f32), (f32, f32)), filling: Filling) {
        self.rect_shader.bind();
        self.rect_shader
            .set_uniform_vec("coord", &cgmath::Vector4::new(x0, y0, x1, y1));
        match filling {
            Filling::Color(r, g, b, a) => {
                self.rect_shader
                    .set_uniform_vec("color", &cgmath::Vector4::new(r, g, b, a));
            }
            Filling::Texture(t, ((tx0, ty0), (tx1, ty1)), (r, g, b)) => {
                t.bind(0);
                self.rect_shader
                    .set_uniform_vec("color", &cgmath::Vector4::new(r, g, b, 0.0));
                self.rect_shader
                    .set_uniform_vec("tex_coord", &cgmath::Vector4::new(tx0, ty0, tx1, ty1));
                self.rect_shader.set_uniform_i("tex", 0);
            }
        }
        self.rect.draw();
    }
}
