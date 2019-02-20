
use crate::model;
use crate::texture;
use crate::asset;
use crate::shader;
use gl::types::*;
use cgmath::prelude::*;

pub struct HeadRenderInfo {
    pub hair: usize,
    pub face: usize,
    pub nose: usize,
    pub beard: usize,
    pub glass: usize,
    pub eye: usize,
    pub eyebrow: usize,
    pub beard_plain: usize,
    pub wrinkle: usize,
    pub makeup: usize,
    pub mole: usize,
    pub lip: usize,
    pub mustache: usize,

    pub full_hair: bool,

    pub hair_color: (f32, f32, f32),
    pub wearing_color: (f32, f32, f32),
    pub face_color: (f32, f32, f32),
    pub beard_color: (f32, f32, f32),
    pub glass_color: (f32, f32, f32),
    pub eye_color: (f32, f32, f32),
    pub eyebrow_color: (f32, f32, f32),
    pub lip_color: (f32, f32, f32),

    pub nose_scale: f32,
    pub nose_y: f32,
    pub glass_y: f32,
    pub glass_scale: f32,
    pub mole_x: f32,
    pub mole_y: f32,
    pub mole_width: f32,
    pub lip_y: f32,
    pub lip_width: f32,
    pub lip_height: f32,
    pub mustache_y: f32,
    pub mustache_width: f32,
    pub mustache_height: f32,
    pub eye_x: f32,
    pub eye_y: f32,
    pub eye_width: f32,
    pub eye_height: f32,
    pub eye_rotation: f32,
    pub eyebrow_x: f32,
    pub eyebrow_y: f32,
    pub eyebrow_width: f32,
    pub eyebrow_height: f32,
    pub eyebrow_rotation: f32,
}

struct TextureWindow {
    tran: cgmath::Matrix3<f32>,
    mirrored: bool
}

struct LayerConfig {
    texture: i32,
    tran: cgmath::Matrix4<f32>,
    window: Option<TextureWindow>,
}

pub struct HeadRenderer {
    asset: asset::Asset,
    head_shader: shader::Shader,
}

impl HeadRenderer {
    pub fn with_asset(asset: asset::Asset) -> HeadRenderer {
        let head_shader = shader::Shader::new(include_str!("head.v.glsl"), include_str!("head.f.glsl"));
        HeadRenderer{asset, head_shader}
    }

    fn set_object_tran(&mut self, object_tran: &cgmath::Matrix4<f32>) {
        let object_tran_inv = object_tran.invert().unwrap();
        self.head_shader.set_uniform_mat4("object_tran", &object_tran);
        self.head_shader.set_uniform_mat4("object_tran_inv", &object_tran_inv);
    }

    fn set_layers (&mut self, configs: [Option<LayerConfig>; 5]) {
        for layer in 0 .. 5 {
            match &configs[layer] {
                None => self.head_shader.set_uniform_mat4(&format!("color_tran[{}]", layer),
                    &cgmath::Matrix4::<f32>::zero()),
                Some(LayerConfig{texture, tran, window}) => {
                    self.head_shader.set_uniform_i(&format!("tex{}", layer), *texture);
                    self.head_shader.set_uniform_mat4(&format!("color_tran[{}]", layer),
                        &tran);
                    let mode_code = match window {
                        None => 0,
                        Some(TextureWindow{tran, mirrored}) => {
                            self.head_shader.set_uniform_mat3(&format!("tex_tran[{}]", layer), &tran);
                            if *mirrored {2} else {1}
                        }
                    };
                    self.head_shader.set_uniform_i(&format!("tex_mode[{}]", layer), mode_code);
                }
            }
        }
    }

    pub fn render_head(&mut self, info: &HeadRenderInfo, object_tran: &cgmath::Matrix4<f32>, aspect: f32) {
        fn draw_model(list: &Vec<Option<model::Model>>, index: usize) {
            list.get(index).and_then(|o|o.as_ref()).map(|m|m.draw());
        };

        fn bind_texture(list: &Vec<Option<texture::Texture>>, index: usize, unit: u32) {
            list.get(index).and_then(|o|o.as_ref()).map(|t|t.bind(unit));
        };

        fn convert_color(color: &(f32, f32, f32), alpha: f32) -> cgmath::Vector4<f32> {
            cgmath::Vector4::new(color.0, color.1, color.2, alpha)
        };

        fn scale4(v: &cgmath::Vector4<f32>) -> cgmath::Matrix4<f32> {
            cgmath::Matrix4::from_cols(
                cgmath::Vector4::new(v.x, 0.0, 0.0, 0.0),
                cgmath::Vector4::new(0.0, v.y, 0.0, 0.0),
                cgmath::Vector4::new(0.0, 0.0, v.z, 0.0),
                cgmath::Vector4::new(0.0, 0.0, 0.0, v.w),
            )
        };

        fn window(p00: (f32, f32), p11: (f32, f32)) -> cgmath::Matrix3<f32> {
            let dx = 1.0 / (p11.0 - p00.0);
            let dy = 1.0 / (p11.1 - p00.1);
            cgmath::Matrix3::from_cols(
                cgmath::Vector3::new(dx, 0.0, 0.0),
                cgmath::Vector3::new(0.0, dy, 0.0),
                cgmath::Vector3::new(-p00.0 * dx, -p00.1 * dy, 1.0)
            )
        }

        fn rotate_around(center: (f32, f32), angle: f32) -> cgmath::Matrix3<f32> {
            cgmath::Matrix3::from_cols(
                cgmath::Vector3::new(1.0, 0.0, 0.0),
                cgmath::Vector3::new(0.0, 1.0, 0.0),
                cgmath::Vector3::new(center.0, center.1, 1.0)
            ) *
            cgmath::Matrix3::from_angle_z(cgmath::Deg(angle)) *
            cgmath::Matrix3::from_cols(
                cgmath::Vector3::new(1.0, 0.0, 0.0),
                cgmath::Vector3::new(0.0, 1.0, 0.0),
                cgmath::Vector3::new(-center.0, -center.1, 1.0)
            )
        }

        let zero_vec4 = cgmath::Vector4::<f32>::zero();

        let face_config = match self.asset.face_configs.get(info.face).and_then(|o|o.as_ref()) {
            Some(f) => f,
            None => return
        };

        let hair_pos = cgmath::Vector3::from(face_config.hair_pos);
        let nose_pos = cgmath::Vector3::from(face_config.nose_pos);
        let beard_pos = cgmath::Vector3::from(face_config.beard_pos);

        self.head_shader.bind();

        // Setup environment
        let camera_pos = cgmath::Point3::new(0.0, 30.0, 400.0);
        let light_source = cgmath::Point3::new(-500.0, 500.0, 500.0);
        let camera_tran = cgmath::perspective(cgmath::Deg(15.0), aspect, 1.0, 1000.0) *
            cgmath::Matrix4::look_at(
            camera_pos,
            cgmath::Point3::new(0.0, 30.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0));
        self.head_shader.set_uniform_vec("camera_pos", &camera_pos);
        self.head_shader.set_uniform_vec("light_source", &light_source);
        self.head_shader.set_uniform_mat4("camera_tran", &camera_tran);

        // Setup blending
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        // bind all relevant textures
        bind_texture(&self.asset.accessory_textures, info.hair, 0);
        bind_texture(&self.asset.eye_textures, info.eye, 1);
        bind_texture(&self.asset.eyebrow_textures, info.eyebrow, 2);
        bind_texture(&self.asset.beard_textures, info.beard_plain, 3);
        bind_texture(&self.asset.wrinkle_textures, info.wrinkle, 4);
        bind_texture(&self.asset.makeup_textures, info.makeup, 5);
        bind_texture(&self.asset.glass_textures, info.glass, 6);
        bind_texture(&self.asset.mole_textures, info.mole, 7);
        bind_texture(&self.asset.lip_textures, info.lip, 8);
        bind_texture(&self.asset.mustache_textures, info.mustache, 9);
        bind_texture(&self.asset.nose_textures, info.nose, 10);

        unsafe {
            gl::Enable(gl::CULL_FACE);
        }

        // Draw face
        self.set_object_tran(&object_tran);
        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.face_color, 1.0));
        // layer order:
        // 0 - makeup
        // 1 - wrinkle
        // 2 - beard
        self.set_layers([
            Some(LayerConfig{texture:5, tran: cgmath::Matrix4::<f32>::identity(), window: None}),
            Some(LayerConfig{texture:4, tran: cgmath::Matrix4::<f32>::identity(), window: None}),
            Some(LayerConfig{texture:3, tran: cgmath::Matrix4::from_cols(
                cgmath::Vector4::zero(),
                cgmath::Vector4::zero(),
                cgmath::Vector4::zero(),
                convert_color(&info.beard_color, 1.0),
                ), window: None}),
            None,
            None,
        ]);

        draw_model(&self.asset.face_models, info.face);

        // Draw hair
        self.set_object_tran(&(object_tran * cgmath::Matrix4::from_translation(hair_pos)));
        let hair_index = info.hair * 2 + if info.full_hair {0} else {1};

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.wearing_color, 1.0));
        self.set_layers([
            Some(LayerConfig{texture:0, tran: scale4(&convert_color(&info.wearing_color, 0.4)), window: None}),
            None, None, None, None,
        ]);
        draw_model(&self.asset.accessory_models, hair_index);

        self.set_layers([None,  None,  None, None, None,]);

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.hair_color, 1.0));
        draw_model(&self.asset.hair_models, hair_index);

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.face_color, 1.0));
        draw_model(&self.asset.scalp_models, hair_index);

        // Draw beard
        self.set_object_tran(&(object_tran * cgmath::Matrix4::from_translation(beard_pos)));
        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.beard_color, 1.0));
        draw_model(&self.asset.beard_models, info.beard);

        // Draw nose model
        let nose_tran = object_tran * cgmath::Matrix4::from_translation(nose_pos
            + cgmath::Vector3::new(0.0, info.nose_y, 0.0)) *
            cgmath::Matrix4::from_scale(info.nose_scale);
        self.set_object_tran(&nose_tran);

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.face_color, 1.0));
        draw_model(&self.asset.nose_models, info.nose);

        // Draw face canvas
        self.set_object_tran(&object_tran);
        self.head_shader.set_uniform_vec("base_color", &zero_vec4);
        // layer order:
        // 0 - mole
        // 1 - eye
        // 2 - eyebrow
        // 3 - lip
        // 4 - mustache
        self.set_layers([
            Some(LayerConfig{texture:7, tran: cgmath::Matrix4::identity(), window: Some(TextureWindow{
                    tran: window((info.mole_x - info.mole_width * 0.5, info.mole_y - info.mole_width * 0.5),
                                 (info.mole_x + info.mole_width * 0.5, info.mole_y + info.mole_width * 0.5)),
                    mirrored: false
                })}),
            Some(LayerConfig{texture:1, tran: cgmath::Matrix4::from_cols(
                    cgmath::Vector4::new(0.0, 1.0, 1.0, 0.0),
                    cgmath::Vector4::new(1.0, 1.0, 1.0, 0.0),
                    convert_color(&info.eye_color, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                ), window: Some(TextureWindow{
                    tran: window((0.5 - info.eye_x - info.eye_width, info.eye_y - info.eye_height * 0.5),
                                 (0.5 - info.eye_x,                  info.eye_y + info.eye_height * 0.5))
                        * rotate_around((0.5 - info.eye_x,info.eye_y), info.eye_rotation),
                    mirrored: true
                })}),
            Some(LayerConfig{texture:2, tran: cgmath::Matrix4::from_cols(
                    cgmath::Vector4::zero(),
                    cgmath::Vector4::zero(),
                    cgmath::Vector4::zero(),
                    convert_color(&info.eyebrow_color, 1.0),
                ), window: Some(TextureWindow{
                    tran: window((0.5 - info.eyebrow_x - info.eyebrow_width, info.eyebrow_y - info.eyebrow_height * 0.5),
                                 (0.5 - info.eyebrow_x,                      info.eyebrow_y + info.eyebrow_height * 0.5))
                        * rotate_around((0.5 - info.eyebrow_x,info.eyebrow_y), info.eyebrow_rotation),
                    mirrored: true
                })}),
            Some(LayerConfig{texture:8, tran: cgmath::Matrix4::from_cols(
                    convert_color(&info.lip_color, 0.0),
                    convert_color(&info.lip_color, 0.0) * 0.5,
                    cgmath::Vector4::new(1.0, 1.0, 1.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                ), window: Some(TextureWindow{
                    tran: window((0.5 - info.lip_width * 0.5, info.lip_y - info.lip_height * 0.5),
                                 (0.5 + info.lip_width * 0.5, info.lip_y + info.lip_height * 0.5)),
                    mirrored: false
                })}),
            Some(LayerConfig{texture:9, tran: cgmath::Matrix4::from_cols(
                    cgmath::Vector4::zero(),
                    cgmath::Vector4::zero(),
                    cgmath::Vector4::zero(),
                    convert_color(&info.beard_color, 1.0),
                ), window: Some(TextureWindow{
                    tran: window((0.5 - info.mustache_width, info.mustache_y - info.mustache_height * 0.5),
                                 (0.5,                       info.mustache_y + info.mustache_height * 0.5)),
                    mirrored: true
                })}),
        ]);

        draw_model(&self.asset.face_canvas_models, info.face);

        // Draw nose canvas
        self.set_object_tran(&nose_tran);

        self.head_shader.set_uniform_vec("base_color", &zero_vec4);
        self.set_layers([
            Some(LayerConfig{texture:10, tran: cgmath::Matrix4::identity(), window: None}),
            None, None, None, None,
        ]);
        draw_model(&self.asset.nose_canvas_models, info.nose);

        // Draw glasses
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }

        self.set_object_tran(&(object_tran * cgmath::Matrix4::from_translation(nose_pos
            + cgmath::Vector3::new(0.0, info.glass_y, 2.0)) *
            cgmath::Matrix4::from_scale(info.glass_scale)));
        self.head_shader.set_uniform_vec("base_color", &zero_vec4);
        self.set_layers([
            Some(LayerConfig{texture:6, tran: scale4(&convert_color(&info.glass_color, 1.0)), window: None}),
            None, None, None, None,
        ]);
        draw_model(&self.asset.glass_models, 0);
    }
}
