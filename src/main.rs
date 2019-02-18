mod shader;
mod texture;
mod model;
mod asset;

use glutin::dpi::*;
use glutin::GlContext;
use gl::types::*;
use cgmath::prelude::*;

struct HeadRenderInfo {
    hair: usize,
    face: usize,
    nose: usize,
    beard: usize,
    glass: usize,
    eye: usize,
    eyebrow: usize,
    beard_plain: usize,
    wrinkle: usize,
    makeup: usize,
    mole: usize,
    lip: usize,
    mustache: usize,

    full_hair: bool,

    hair_color: (f32, f32, f32),
    wearing_color: (f32, f32, f32),
    face_color: (f32, f32, f32),
    beard_color: (f32, f32, f32),
    glass_color: (f32, f32, f32),
    eye_color: (f32, f32, f32),
    lip_color: (f32, f32, f32),
}

struct HeadRenderer {
    asset: asset::Asset,
    head_shader: shader::Shader,
}

impl HeadRenderer {
    fn with_asset(asset: asset::Asset) -> HeadRenderer {
        let head_shader = shader::Shader::new(include_str!("head.v.glsl"), include_str!("head.f.glsl"));
        HeadRenderer{asset, head_shader}
    }

    fn render_head(&mut self, info: &HeadRenderInfo, object_tran: &cgmath::Matrix4<f32>, aspect: f32) {
        let set_object_tran = |shader: &mut shader::Shader, object_tran: &cgmath::Matrix4<f32>| {
            let object_tran_inv = object_tran.invert().unwrap();
            shader.set_uniform_mat4("object_tran", &object_tran);
            shader.set_uniform_mat4("object_tran_inv", &object_tran_inv);
        };

        let draw_model = |list: &Vec<Option<model::Model>>, index: usize| {
            list.get(index).and_then(|o|o.as_ref()).map(|m|m.draw());
        };

        let bind_texture = |list: &Vec<Option<texture::Texture>>, index: usize, unit: u32| {
            list.get(index).and_then(|o|o.as_ref()).map(|t|t.bind(unit));
        };

        let convert_color = |color: &(f32, f32, f32), alpha: f32| -> cgmath::Vector4<f32> {
            cgmath::Vector4::new(color.0, color.1, color.2, alpha)
        };

        let scale4 = |v: &cgmath::Vector4<f32>| -> cgmath::Matrix4<f32> {
            cgmath::Matrix4::from_cols(
                cgmath::Vector4::new(v.x, 0.0, 0.0, 0.0),
                cgmath::Vector4::new(0.0, v.y, 0.0, 0.0),
                cgmath::Vector4::new(0.0, 0.0, v.z, 0.0),
                cgmath::Vector4::new(0.0, 0.0, 0.0, v.w),
            )
        };

        struct TextureWindow {
            min: cgmath::Point2<f32>,
            max: cgmath::Point2<f32>,
            mirrored: bool
        }

        struct LayerConfig {
            texture: i32,
            tran: cgmath::Matrix4<f32>,
            window: Option<TextureWindow>,
        }

        let set_layers = |shader: &mut shader::Shader, configs: [Option<LayerConfig>; 5]| {
            for layer in 0 .. 5 {
                match &configs[layer] {
                    None => shader.set_uniform_mat4(&format!("color_tran[{}]", layer),
                        &cgmath::Matrix4::<f32>::zero()),
                    Some(LayerConfig{texture, tran, window}) => {
                        shader.set_uniform_i(&format!("tex{}", layer), *texture);
                        shader.set_uniform_mat4(&format!("color_tran[{}]", layer),
                            &tran);
                        let mode_code = match window {
                            None => 0,
                            Some(TextureWindow{min, max, mirrored}) => {
                                shader.set_uniform_vec(&format!("tex_window[{}]", layer),
                                    &cgmath::Vector4::new(min.x, min.y, max.x, max.y));
                                if *mirrored {2} else {1}
                            }
                        };
                        shader.set_uniform_i(&format!("tex_mode[{}]", layer), mode_code);
                    }
                }
            }
        };

        let zero_vec4 = cgmath::Vector4::<f32>::zero();

        let face_config = match self.asset.face_configs.get(info.face).and_then(|o|o.as_ref()) {
            Some(f) => f,
            None => return
        };

        self.head_shader.bind();

        // Setup environment
        let camera_pos = cgmath::Point3::new(0.0, 30.0, 120.0);
        let light_source = cgmath::Point3::new(500.0, 500.0, 500.0);
        let camera_tran = cgmath::perspective(cgmath::Deg(45.0), aspect, 1.0, 1000.0) *
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
        set_object_tran(&mut self.head_shader, &object_tran);
        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.face_color, 1.0));
        // layer order:
        // 0 - makeup
        // 1 - wrinkle
        // 2 - beard
        set_layers(&mut self.head_shader, [
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
        set_object_tran(&mut self.head_shader,
            &(object_tran * cgmath::Matrix4::from_translation(
            cgmath::Vector3::from(face_config.hair_pos))));
        let hair_index = info.hair * 2 + if info.full_hair {0} else {1};

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.wearing_color, 1.0));
        set_layers(&mut self.head_shader, [
            Some(LayerConfig{texture:0, tran: scale4(&convert_color(&info.wearing_color, 0.4)), window: None}),
            None,
            None,
            None,
            None,
        ]);
        draw_model(&self.asset.accessory_models, hair_index);

        set_layers(&mut self.head_shader, [
            None,
            None,
            None,
            None,
            None,
        ]);

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.hair_color, 1.0));
        draw_model(&self.asset.hair_models, hair_index);

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.face_color, 1.0));
        draw_model(&self.asset.scalp_models, hair_index);

        // Draw beard
        set_object_tran(&mut self.head_shader,
            &(object_tran * cgmath::Matrix4::from_translation(
            cgmath::Vector3::from(face_config.beard_pos))));
        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.beard_color, 1.0));
        draw_model(&self.asset.beard_models, info.beard);

        // Draw nose model
        set_object_tran(&mut self.head_shader,
            &(object_tran * cgmath::Matrix4::from_translation(
            cgmath::Vector3::from(face_config.nose_pos))));

        self.head_shader.set_uniform_vec("base_color", &convert_color(&info.face_color, 1.0));
        draw_model(&self.asset.nose_models, info.nose);

        // Draw face canvas
        set_object_tran(&mut self.head_shader, &object_tran);
        self.head_shader.set_uniform_vec("base_color", &zero_vec4);
        // layer order:
        // 0 - mole
        // 1 - eye
        // 2 - eyebrow
        // 3 - lip
        // 4 - mustache
        set_layers(&mut self.head_shader, [
            Some(LayerConfig{texture:7, tran: cgmath::Matrix4::identity(), window: Some(TextureWindow{
                    min: cgmath::Point2::new(0.35, 0.35), max: cgmath::Point2::new(0.4, 0.4), mirrored: false
                })}),
            Some(LayerConfig{texture:1, tran: cgmath::Matrix4::from_cols(
                    cgmath::Vector4::new(0.0, 1.0, 1.0, 0.0),
                    cgmath::Vector4::new(1.0, 1.0, 1.0, 0.0),
                    convert_color(&info.eye_color, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                ), window: Some(TextureWindow{
                    min: cgmath::Point2::new(0.30, 0.45), max: cgmath::Point2::new(0.48, 0.6), mirrored: true
                })}),
            Some(LayerConfig{texture:2, tran: cgmath::Matrix4::identity(), window: Some(TextureWindow{
                    min: cgmath::Point2::new(0.30, 0.5), max: cgmath::Point2::new(0.47, 0.65), mirrored: true
                })}),
            Some(LayerConfig{texture:8, tran: cgmath::Matrix4::from_cols(
                    convert_color(&info.lip_color, 0.0),
                    convert_color(&info.lip_color, 0.0) * 0.5,
                    cgmath::Vector4::new(1.0, 1.0, 1.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                ), window: Some(TextureWindow{
                    min: cgmath::Point2::new(0.43, 0.3), max: cgmath::Point2::new(0.57, 0.4), mirrored: false
                })}),
            Some(LayerConfig{texture:9, tran: cgmath::Matrix4::identity(), window: Some(TextureWindow{
                    min: cgmath::Point2::new(0.4, 0.27), max: cgmath::Point2::new(0.5, 0.47), mirrored: true
                })}),
        ]);

        draw_model(&self.asset.face_canvas_models, info.face);

        // Draw nose canvas
        set_object_tran(&mut self.head_shader,
            &(object_tran * cgmath::Matrix4::from_translation(
            cgmath::Vector3::from(face_config.nose_pos))));

        self.head_shader.set_uniform_vec("base_color", &zero_vec4);
        set_layers(&mut self.head_shader, [
            Some(LayerConfig{texture:10, tran: cgmath::Matrix4::identity(), window: None}),
            None,
            None,
            None,
            None,
        ]);
        draw_model(&self.asset.nose_canvas_models, info.nose);

        // Draw glasses
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }

        set_object_tran(&mut self.head_shader,
            &(object_tran * cgmath::Matrix4::from_translation(
            cgmath::Vector3::from(face_config.nose_pos) + cgmath::Vector3::new(0.0, 5.0, 2.0))));
        self.head_shader.set_uniform_vec("base_color", &zero_vec4);
        set_layers(&mut self.head_shader, [
            Some(LayerConfig{texture:6, tran: scale4(&convert_color(&info.glass_color, 1.0)), window: None}),
            None,
            None,
            None,
            None,
        ]);
        draw_model(&self.asset.glass_models, 0);
    }
}

extern "system"
fn gl_debug_message(_source: GLenum, _type: GLenum, _id: GLuint, sev: GLenum,
                    len: GLsizei, message: *const GLchar,
                    _param: *mut GLvoid) {
    if sev != gl::DEBUG_SEVERITY_HIGH && sev != gl::DEBUG_SEVERITY_MEDIUM {
        return;
    }
    unsafe {
        let s = std::str::from_utf8(std::slice::from_raw_parts(message as *const u8, len as usize)).unwrap();
        println!("OpenGL: {}", s);
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Citrii")
        .with_dimensions(LogicalSize::new(500.0, 500.0));
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(gl_debug_message, std::ptr::null_mut());

        gl::ClearColor(0.5, 1.0, 0.5, 1.0);
        gl::ClearDepth(1.0);
        gl::Enable(gl::DEPTH_TEST);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: citrii [Path to CFL_Res.dat]");
        return;
    }
    let asset_data = std::fs::read(&args[1]).expect("Unable to read CFL_Res.dat");
    let asset = asset::Asset::from_bytes(&asset_data).expect("The provided CFL_Res.dat is corrupted");
    let mut head_renderer = HeadRenderer::with_asset(asset);

    let mut rotate = 0.0;
    let mut aspect = 1.0;
    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = gl_window.get_hidpi_factor();
                        let physical = logical_size.to_physical(dpi_factor);
                        aspect = (physical.width / physical.height) as f32;
                        unsafe {
                            gl::Viewport(0, 0, physical.width as GLint, physical.height as GLint);
                        }
                        gl_window.resize(physical);
                    },
                    glutin::WindowEvent::KeyboardInput{input, ..} => {
                        match (input.state, input.virtual_keycode) {
                            (glutin::ElementState::Pressed, Some(glutin::VirtualKeyCode::Left)) => {
                                rotate -= 10.0;
                            },
                            (glutin::ElementState::Pressed, Some(glutin::VirtualKeyCode::Right)) => {
                                rotate += 10.0;
                            },
                            _ => ()
                        }
                    }
                    _ => ()
                },
                _ => ()
            }
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let info = HeadRenderInfo {
            hair: 34,
            face: 0,
            nose: 1,
            beard: 0,
            glass: 8,
            eye: 0,
            eyebrow: 0,
            beard_plain: 1,
            wrinkle: 11,
            makeup: 6,
            mole: 1,
            lip: 29,
            mustache: 3,

            full_hair: true,

            hair_color: (0.1, 0.1, 0.1),
            wearing_color: (1.0, 0.1, 0.1),
            face_color: (1.0, 0.85, 0.6),
            beard_color: (0.2, 0.2, 0.0),
            glass_color: (0.8, 0.0, 1.0),
            eye_color: (0.0, 1.0, 0.0),
            lip_color: (1.0, 0.2, 0.2),
        };

        let object_tran = cgmath::Matrix4::from_angle_y(cgmath::Deg(rotate));

        head_renderer.render_head(&info, &object_tran, aspect);

        gl_window.swap_buffers().unwrap();
    }
}
