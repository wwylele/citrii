use crate::rect_renderer;
use crate::texture;

pub struct TextRenderer {
    rect_renderer: std::rc::Rc<rect_renderer::RectRenderer>,
    font: texture::Texture,
    font_width: usize,
    font_height: usize,
}

impl TextRenderer {
    pub fn new(rect_renderer: std::rc::Rc<rect_renderer::RectRenderer>) -> TextRenderer {
        if let Ok(image::DynamicImage::ImageRgba8(image_buffer))
            = image::load_from_memory(include_bytes!("font_fira_code.PNG")) {
            let width = image_buffer.width() as usize;
            let height = image_buffer.height() as usize;
            let font = texture::Texture::new(width, height, &image_buffer.into_raw(),
                &texture::WrapMode::Edge, &texture::WrapMode::Edge);
            TextRenderer {
                rect_renderer,
                font,
                font_width: width,
                font_height: height
            }
        } else {
            panic!("broken font image");
        }
    }

    pub fn render(&self, text: &str, (x, y): (f32, f32), height: f32, color: (f32, f32, f32), aspect: f32) {
        const FONT_X_COUNT: u8 = 0x5F;
        const FONT_WIDTH: f32 = 1.0 / FONT_X_COUNT as f32;

        let y_min = y - height * 0.5;
        let y_max = y + height * 0.5;
        let char_width = height / (self.font_height as f32) * (FONT_WIDTH * self.font_width as f32) / aspect;
        let width = char_width * text.len() as f32;
        let mut x_min = x - width * 0.5;

        for c in text.chars() {
            let code: u8 = if c.len_utf8() == 1 || c.is_ascii_control() {
                let mut buf = [0];
                c.encode_utf8(&mut buf);
                buf[0]
            } else {
                '?' as u8
            } - 0x20;

            let tex_x = (code % FONT_X_COUNT) as f32 * FONT_WIDTH;
            self.rect_renderer.render(((x_min, y_min), (x_min + char_width, y_max)),
                rect_renderer::Filling::Texture(&self.font, ((tex_x, 1.0), (tex_x + FONT_WIDTH, 0.0)), color));
            x_min += char_width;
        }

    }
}
