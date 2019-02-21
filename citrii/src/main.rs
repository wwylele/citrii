#![allow(dead_code)]

mod shader;
mod texture;
mod model;
mod asset;
mod database;
mod head_renderer;
mod crc;

use std::io::Write;

use glutin::dpi::*;
use glutin::GlContext;
use gl::types::*;
use cgmath::prelude::*;

use byte_struct::*;

const PAGE_FACE: u8 = 0;
const PAGE_MAKEUP: u8 = 1;
const PAGE_WRINKLE: u8 = 2;
const PAGE_HAIR: u8 = 3;
const PAGE_EYEBROW: u8 = 4;
const PAGE_EYE: u8 = 5;
const PAGE_NOSE: u8 = 6;
const PAGE_LIP: u8 = 7;
const PAGE_GLASS: u8 = 8;
const PAGE_MUSTACHE: u8 = 9;
const PAGE_MOLE: u8 = 10;
const PAGE_BEARD: u8 = 11;

const PAGE_END: u8 = 12;

enum Delta {
    Inc,
    Dec,
}

fn wrap_change_value<T: std::ops::AddAssign<T> + std::ops::SubAssign<T> + std::cmp::PartialOrd<T> + From<u8>>
    (v: &mut T, limit: T, delta: Delta) {
    match delta {
        Delta::Inc => {
            *v += T::from(1u8);
            if *v >= limit {
                *v = T::from(0u8);
            }
        },
        Delta::Dec => {
            if *v <= T::from(0u8) {
                *v = limit;
            }
            *v -= T::from(1u8);
        }
    }
}

fn clamp_change_value<T: std::ops::AddAssign<T> + std::ops::SubAssign<T> + std::cmp::PartialOrd<T> + From<u8>>
    (v: &mut T, min: T, max: T, delta: Delta) {
    match delta {
        Delta::Inc => {
            if *v >= max {
                return
            }
            *v += T::from(1u8);
        },
        Delta::Dec => {
            if *v <= min {
                return
            }
            *v -= T::from(1u8);
        }
    }
}


struct Main {
    gl_window: glutin::GlWindow,

    database_filename: String,

    head_renderer: head_renderer::HeadRenderer,
    database: database::Database,

    profile_index: usize,
    page: u8,
}

impl Main {
    fn new(asset_filename: &str, database_filename: &str, events_loop: &mut glutin::EventsLoop) -> Main {
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

        let asset_data = std::fs::read(asset_filename).expect("Unable to read CFL_Res.dat");
        let asset = asset::Asset::from_bytes(&asset_data).expect("The provided CFL_Res.dat is corrupted");
        let head_renderer = head_renderer::HeadRenderer::with_asset(asset);

        let database_data = std::fs::read(database_filename).expect("Unable to read CFL_Res.dat");
        let crc_a = crc::crc16_ninty(&database_data[0 .. 0xC81E]);
        assert_eq!(crc_a, u16::from_be_bytes([database_data[0xC81E], database_data[0xC81F]]));
        let database = database::Database::read_bytes(&database_data[..]);

        Main {
            gl_window,
            database_filename: String::from(database_filename),
            head_renderer,
            database,
            profile_index: 0,
            page: 0
        }
    }

    fn on_style_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_FACE => wrap_change_value(&mut profile.face.style, 12, delta),
            PAGE_MAKEUP => wrap_change_value(&mut profile.face.makeup, 12, delta),
            PAGE_WRINKLE => wrap_change_value(&mut profile.face.wrinkle, 12, delta),
            PAGE_HAIR => wrap_change_value(&mut profile.hair.style, 132, delta),
            PAGE_EYEBROW => wrap_change_value(&mut profile.eyebrow.style, 24, delta),
            PAGE_EYE => wrap_change_value(&mut profile.eye.style, 62, delta),
            PAGE_NOSE => wrap_change_value(&mut profile.nose.style, 18, delta),
            PAGE_LIP => wrap_change_value(&mut profile.lip.style, 37, delta),
            PAGE_GLASS => wrap_change_value(&mut profile.glass.style, 9, delta),
            PAGE_MUSTACHE => wrap_change_value(&mut profile.misc.mustache_style, 6, delta),
            PAGE_MOLE => wrap_change_value(&mut profile.mole.style, 2, delta),
            PAGE_BEARD => wrap_change_value(&mut profile.beard.style, 6, delta),
            _ => (),
        }
    }

    fn on_color_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_FACE => wrap_change_value(&mut profile.face.color, 6, delta),
            PAGE_HAIR => wrap_change_value(&mut profile.hair.color, 8, delta),
            PAGE_EYEBROW => wrap_change_value(&mut profile.eyebrow.color, 8, delta),
            PAGE_EYE => wrap_change_value(&mut profile.eye.color, 6, delta),
            PAGE_LIP => wrap_change_value(&mut profile.lip.color, 5, delta),
            PAGE_GLASS => wrap_change_value(&mut profile.glass.color, 6, delta),
            // mustache and beard share the same color
            PAGE_MUSTACHE => wrap_change_value(&mut profile.beard.color, 8, delta),
            PAGE_BEARD => wrap_change_value(&mut profile.beard.color, 8, delta),
            _ => (),
        }
    }

    fn on_scale_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.scale, 0, 8, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.scale, 0, 7, delta),
            PAGE_NOSE => clamp_change_value(&mut profile.nose.scale, 0, 8, delta),
            PAGE_LIP => clamp_change_value(&mut profile.lip.scale, 0, 8, delta),
            PAGE_GLASS => clamp_change_value(&mut profile.glass.scale, 0, 7, delta),
            PAGE_MUSTACHE => clamp_change_value(&mut profile.beard.mustache_scale, 0, 8, delta),
            PAGE_MOLE => clamp_change_value(&mut profile.mole.scale, 0, 8, delta),
            _ => (),
        }
    }

    fn on_y_scale_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.y_scale, 0, 6, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.y_scale, 0, 6, delta),
            PAGE_LIP => clamp_change_value(&mut profile.lip.y_scale, 0, 6, delta),
            _ => (),
        }
    }

    fn on_rotation_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.rotation, 0, 11, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.rotation, 0, 7, delta),
            _ => (),
        }
    }

    fn on_x_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.x, 0, 12, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.x, 0, 12, delta),
            PAGE_MOLE => clamp_change_value(&mut profile.mole.x, 0, 0x10, delta),
            _ => (),
        }
    }

    fn on_y_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index];
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.y, 3, 18, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.y, 0, 18, delta),
            PAGE_NOSE => clamp_change_value(&mut profile.nose.y, 0, 18, delta),
            PAGE_LIP => clamp_change_value(&mut profile.misc.lip_y, 0, 18, delta),
            PAGE_GLASS => clamp_change_value(&mut profile.glass.y, 0, 20, delta),
            PAGE_MUSTACHE => clamp_change_value(&mut profile.beard.mustache_y, 0, 16, delta),
            PAGE_MOLE => clamp_change_value(&mut profile.mole.y, 0, 30, delta),
            _ => (),
        }
    }

    fn on_save(&self) {
        let mut database_data = vec![0u8; database::Database::byte_len()];
        self.database.write_bytes(&mut database_data[..]);
        let crc_a = crc::crc16_ninty(&database_data[0 .. 0xC81E]).to_be_bytes();
        database_data[0xC81E .. 0xC820].copy_from_slice(&crc_a);
        // TODO: write the entire file directly once the database struct is complete
        let file = std::fs::OpenOptions::new().write(true).open(&self.database_filename);
        if file.is_err() {
            println!("Failed to open file");
            return
        }
        if file.unwrap().write_all(&database_data[..]).is_err() {
            println!("Failed to write file");
            return
        }
        println!("Saved");
    }

    fn run (&mut self, events_loop: &mut glutin::EventsLoop) {
        let mut rotate = 0.0;
        let mut aspect = 1.0;
        let mut running = true;
        while running {
            events_loop.poll_events(|event| {
                match event {
                    glutin::Event::WindowEvent{ event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => running = false,
                        glutin::WindowEvent::Resized(logical_size) => {
                            let dpi_factor = self.gl_window.get_hidpi_factor();
                            let physical = logical_size.to_physical(dpi_factor);
                            aspect = (physical.width / physical.height) as f32;
                            unsafe {
                                gl::Viewport(0, 0, physical.width as GLint, physical.height as GLint);
                            }
                            self.gl_window.resize(physical);
                        },
                        glutin::WindowEvent::KeyboardInput{
                            input: glutin::KeyboardInput{state: glutin::ElementState::Pressed, virtual_keycode, ..},
                        ..} => {
                            match virtual_keycode {
                                Some(glutin::VirtualKeyCode::A) => {
                                    rotate += 10.0;
                                },
                                Some(glutin::VirtualKeyCode::D) => {
                                    rotate -= 10.0;
                                },

                                Some(glutin::VirtualKeyCode::Up) => {
                                    if self.page == 0 {
                                        self.page = PAGE_END;
                                    }
                                    self.page -= 1;
                                    println!("page = {}", self.page);
                                },

                                Some(glutin::VirtualKeyCode::Down) => {
                                    self.page += 1;
                                    if self.page == PAGE_END {
                                        self.page = 0;
                                    }
                                    println!("page = {}", self.page);
                                },

                                Some(glutin::VirtualKeyCode::Left) => {
                                    self.on_style_change(Delta::Dec);
                                }

                                Some(glutin::VirtualKeyCode::Right) => {
                                    self.on_style_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::F) => {
                                    self.on_x_change(Delta::Dec);
                                }
                                Some(glutin::VirtualKeyCode::H) => {
                                    self.on_x_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::T) => {
                                    self.on_y_change(Delta::Dec);
                                }
                                Some(glutin::VirtualKeyCode::G) => {
                                    self.on_y_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::R) => {
                                    self.on_rotation_change(Delta::Dec);
                                }
                                Some(glutin::VirtualKeyCode::Y) => {
                                    self.on_rotation_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::Z) => {
                                    self.on_color_change(Delta::Dec);
                                }
                                Some(glutin::VirtualKeyCode::X) => {
                                    self.on_color_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::C) => {
                                    self.on_scale_change(Delta::Dec);
                                }
                                Some(glutin::VirtualKeyCode::V) => {
                                    self.on_scale_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::B) => {
                                    self.on_y_scale_change(Delta::Dec);
                                }
                                Some(glutin::VirtualKeyCode::N) => {
                                    self.on_y_scale_change(Delta::Inc);
                                }

                                Some(glutin::VirtualKeyCode::Return) => {
                                    self.on_save();
                                }

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

            let info = self.database.owned[self.profile_index].to_render_info();

            let object_tran = cgmath::Matrix4::from_angle_y(cgmath::Deg(rotate));

            self.head_renderer.render_head(&info, &object_tran, aspect);

            self.gl_window.swap_buffers().unwrap();
        }
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        panic!("Usage: citrii [Path to CFL_Res.dat] [Path to CFL_DB.dat]");
    }
    let mut events_loop = glutin::EventsLoop::new();
    let mut instance = Main::new(&args[1], &args[2], &mut events_loop);
    instance.run(&mut events_loop);
}
