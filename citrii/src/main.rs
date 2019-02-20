mod shader;
mod texture;
mod model;
mod asset;
mod database;
mod head_renderer;

use glutin::dpi::*;
use glutin::GlContext;
use gl::types::*;
use cgmath::prelude::*;

use byte_struct::*;

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

        if gl::DebugMessageCallback::is_loaded() {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(gl_debug_message, std::ptr::null_mut());
        }

        gl::ClearColor(0.5, 1.0, 0.5, 1.0);
        gl::ClearDepth(1.0);
        gl::Enable(gl::DEPTH_TEST);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: citrii [Path to CFL_Res.dat] [Path to CFL_DB.dat]");
        return;
    }
    let asset_data = std::fs::read(&args[1]).expect("Unable to read CFL_Res.dat");
    let asset = asset::Asset::from_bytes(&asset_data).expect("The provided CFL_Res.dat is corrupted");
    let mut head_renderer = head_renderer::HeadRenderer::with_asset(asset);

    let database_data = std::fs::read(&args[2]).expect("Unable to read CFL_Res.dat");
    let database = database::Database::read_bytes(&database_data[..]);
    println!("{:#?}", database.owned[0]);

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

        /*let info = head_renderer::HeadRenderInfo {
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
            eyebrow_color: (0.5, 0.5, 0.5),
            lip_color: (1.0, 0.2, 0.2),
        };*/
        let info = database.owned[0].to_render_info();

        let object_tran = cgmath::Matrix4::from_angle_y(cgmath::Deg(rotate));

        head_renderer.render_head(&info, &object_tran, aspect);

        gl_window.swap_buffers().unwrap();
    }
}
