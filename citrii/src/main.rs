#![allow(dead_code)]

mod shader;
mod texture;
mod model;
mod asset;
mod database;
mod head_renderer;
mod rect_renderer;
mod text_renderer;
mod crc;
mod ui;
mod color;

use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;

use glutin::dpi::*;
use glutin::GlContext;
use gl::types::*;

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
const PAGE_BEARD: u8 = 10;
const PAGE_MOLE: u8 = 11;

const PAGE_END: u8 = 12;

const PAGE_NAMES: [&str; 12] = [
    "Face",
    "Makeup",
    "Wrinkle",
    "Hairstyle",
    "Eyebrows",
    "Eyes",
    "Nose",
    "Mouth",
    "Glasses",
    "Mustache",
    "Goatee",
    "Beauty mark",
];

const ID_PAGE_BUTTON_BEGIN: u32 = 100;
const ID_PAGE_BUTTON_END: u32 = ID_PAGE_BUTTON_BEGIN + PAGE_END as u32;
const ID_STYLE_DEC: u32 = 200;
const ID_STYLE_INC: u32 = 201;
const ID_Y_DEC: u32 = 202;
const ID_Y_INC: u32 = 203;
const ID_X_DEC: u32 = 204;
const ID_X_INC: u32 = 205;
const ID_ROTATION_DEC: u32 = 206;
const ID_ROTATION_INC: u32 = 207;
const ID_SCALE_DEC: u32 = 208;
const ID_SCALE_INC: u32 = 209;
const ID_Y_SCALE_DEC: u32 = 210;
const ID_Y_SCALE_INC: u32 = 211;
const ID_PALETTE: u32 = 300;

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
    rect_renderer: std::rc::Rc<rect_renderer::RectRenderer>,
    text_renderer: std::rc::Rc<text_renderer::TextRenderer>,
    database: database::Database,

    profile_index: usize,
    page: u8,

    scene: ui::Scene,
    page_buttons: Vec<Rc<RefCell<ui::Button>>>,
    palette: Rc<RefCell<ui::Palette>>,
    button_y_dec: Rc<RefCell<ui::Button>>,
    button_y_inc: Rc<RefCell<ui::Button>>,
    button_x_dec: Rc<RefCell<ui::Button>>,
    button_x_inc: Rc<RefCell<ui::Button>>,
    button_x_space_dec: Rc<RefCell<ui::Button>>,
    button_x_space_inc: Rc<RefCell<ui::Button>>,
    button_rotation_dec: Rc<RefCell<ui::Button>>,
    button_rotation_inc: Rc<RefCell<ui::Button>>,
    button_scale_dec: Rc<RefCell<ui::Button>>,
    button_scale_inc: Rc<RefCell<ui::Button>>,
    button_y_scale_dec: Rc<RefCell<ui::Button>>,
    button_y_scale_inc: Rc<RefCell<ui::Button>>,
}

impl Main {
    fn new(asset_filename: &str, database_filename: &str, events_loop: &mut glutin::EventsLoop) -> Main {
        let window = glutin::WindowBuilder::new()
            .with_title("Citrii")
            .with_dimensions(LogicalSize::new(800.0, 600.0));
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
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let asset_data = std::fs::read(asset_filename).expect("Unable to read CFL_Res.dat");
        let asset = asset::Asset::from_bytes(&asset_data).expect("The provided CFL_Res.dat is corrupted");
        let head_renderer = head_renderer::HeadRenderer::with_asset(asset);

        let database_data = std::fs::read(database_filename).expect("Unable to read CFL_Res.dat");
        let crc_a = crc::crc16_ninty(&database_data[0 .. 0xC81E]);
        assert_eq!(crc_a, u16::from_be_bytes([database_data[0xC81E], database_data[0xC81F]]));
        let database = database::Database::read_bytes(&database_data[..]);

        let rect_renderer = std::rc::Rc::new(rect_renderer::RectRenderer::new());
        let text_renderer = std::rc::Rc::new(text_renderer::TextRenderer::new(rect_renderer.clone()));

        let mut page_buttons: Vec<Rc<RefCell<ui::Button>>> = vec![];

        for i in 0 .. PAGE_END {
            let button = ui::Button::new(
                ID_PAGE_BUTTON_BEGIN + i as u32,
                0.35, 0.05, ui::ButtonContent::from_text(PAGE_NAMES[i as usize]),
                rect_renderer.clone(),
                text_renderer.clone());
            page_buttons.push(button);
        }

        let layout_pages = ui::GridLayout::new(1, PAGE_END as usize,
            page_buttons.iter().map(|b|-> Rc<RefCell<dyn ui::UIElement>> {b.clone()}).collect(),
            0.02, 0.02, 0.02, 0.02, 0.01, 0.01);
        let docker_pages = ui::Docker::new(layout_pages, ui::XAlign::Left, ui::YAlign::Top);

        let button_style_dec = ui::Button::new(ID_STYLE_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/previous.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_style_inc = ui::Button::new(ID_STYLE_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/next.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_y_dec = ui::Button::new(ID_Y_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/down.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_y_inc = ui::Button::new(ID_Y_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/up.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_x_dec = ui::Button::new(ID_X_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/left.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_x_inc = ui::Button::new(ID_X_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/right.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_x_space_dec = ui::Button::new(ID_X_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/x-scale-down.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_x_space_inc = ui::Button::new(ID_X_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/x-scale-up.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_rotation_dec = ui::Button::new(ID_ROTATION_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/rotate-up.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_rotation_inc = ui::Button::new(ID_ROTATION_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/rotate-down.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_scale_dec = ui::Button::new(ID_SCALE_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/scale-down.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_scale_inc = ui::Button::new(ID_SCALE_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/scale-up.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_y_scale_dec = ui::Button::new(ID_Y_SCALE_DEC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/y-scale-down.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_y_scale_inc = ui::Button::new(ID_Y_SCALE_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/y-scale-up.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let layout_controls = ui::GridLayout::new(2, 7, vec![
            button_style_dec, button_style_inc,
            button_y_dec.clone(), button_y_inc.clone(),
            button_x_dec.clone(), button_x_inc.clone(),
            button_x_space_dec.clone(), button_x_space_inc.clone(),
            button_y_scale_inc.clone(), button_y_scale_dec.clone(),
            button_rotation_dec.clone(), button_rotation_inc.clone(),
            button_scale_inc.clone(), button_scale_dec.clone(),
        ], 0.02, 0.02, 0.02, 0.02, 0.01, 0.01);

        let palette = ui::Palette::new(ID_PALETTE, 0.07, rect_renderer.clone());

        let layout_controls_ex = ui::GridLayout::new(2, 1, vec![
            palette.clone(), layout_controls
        ], 0.02, 0.02, 0.02, 0.02, 0.01, 0.01);

        let docker_controls = ui::Docker::new(layout_controls_ex, ui::XAlign::Right, ui::YAlign::Top);

        let scene = ui::Scene::new(vec![docker_pages, docker_controls]);

        Main {
            gl_window,
            database_filename: String::from(database_filename),
            head_renderer,
            rect_renderer,
            text_renderer,
            database,
            profile_index: 0,
            page: 0,
            scene,
            page_buttons,
            palette,
            button_y_dec,
            button_y_inc,
            button_x_dec,
            button_x_inc,
            button_x_space_dec,
            button_x_space_inc,
            button_rotation_dec,
            button_rotation_inc,
            button_scale_dec,
            button_scale_inc,
            button_y_scale_dec,
            button_y_scale_inc,
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
            PAGE_MUSTACHE | PAGE_BEARD => wrap_change_value(&mut profile.beard.color, 8, delta),
            _ => (),
        }
    }

    fn on_color_change_from_palette(&mut self) {
        let profile = &mut self.database.owned[self.profile_index];
        let color = self.palette.borrow().get_selected();
        match self.page {
            PAGE_FACE => profile.face.color = color as u16,
            PAGE_HAIR => profile.hair.color = color as u16,
            PAGE_EYEBROW => profile.eyebrow.color = color as u32,
            PAGE_EYE => profile.eye.color = color as u32,
            PAGE_LIP => profile.lip.color = color as u16,
            PAGE_GLASS => profile.glass.color = color as u16,
            PAGE_MUSTACHE | PAGE_BEARD  => profile.beard.color = color as u16,
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

    fn on_page_change(&mut self, page: u8) {
        self.page = page;
        for i in 0 .. PAGE_END {
            self.page_buttons[i as usize].borrow_mut().set_selected(i == page);
        }

        let enable_y = page == PAGE_EYEBROW || page == PAGE_EYE ||
                       page == PAGE_NOSE || page == PAGE_LIP ||
                       page == PAGE_GLASS || page == PAGE_MUSTACHE || page == PAGE_MOLE;
        let enable_x = page == PAGE_MOLE;
        let enable_x_space = page == PAGE_EYEBROW || page == PAGE_EYE;
        let enable_rotation = page == PAGE_EYEBROW || page == PAGE_EYE;
        let enable_scale = enable_y;
        let enable_y_scale = page == PAGE_EYEBROW || page == PAGE_EYE || page == PAGE_LIP;

        self.button_y_dec.borrow_mut().set_visible(enable_y);
        self.button_y_inc.borrow_mut().set_visible(enable_y);
        self.button_x_dec.borrow_mut().set_visible(enable_x);
        self.button_x_inc.borrow_mut().set_visible(enable_x);
        self.button_x_space_dec.borrow_mut().set_visible(enable_x_space);
        self.button_x_space_inc.borrow_mut().set_visible(enable_x_space);
        self.button_rotation_dec.borrow_mut().set_visible(enable_rotation);
        self.button_rotation_inc.borrow_mut().set_visible(enable_rotation);
        self.button_scale_dec.borrow_mut().set_visible(enable_scale);
        self.button_scale_inc.borrow_mut().set_visible(enable_scale);
        self.button_y_scale_dec.borrow_mut().set_visible(enable_y_scale);
        self.button_y_scale_inc.borrow_mut().set_visible(enable_y_scale);

        let profile = &self.database.owned[self.profile_index];
        let mut palette = self.palette.borrow_mut();
        match page {
            PAGE_FACE => {
                palette.set_colors(color::SKIN_COLOR_TABLE.to_vec());
                palette.set_selected(profile.face.color as usize);
            },
            PAGE_MAKEUP => {
                palette.set_colors(vec![]);
            },
            PAGE_WRINKLE => {
                palette.set_colors(vec![]);
            },
            PAGE_HAIR => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.hair.color as usize);
            },
            PAGE_EYEBROW => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.eyebrow.color as usize);
            },
            PAGE_EYE => {
                palette.set_colors(color::EYE_COLOR_TABLE.to_vec());
                palette.set_selected(profile.eye.color as usize);
            },
            PAGE_NOSE => {
                palette.set_colors(vec![]);
            },
            PAGE_LIP => {
                palette.set_colors(color::LIP_COLOR_TABLE.to_vec());
                palette.set_selected(profile.lip.color as usize);
            },
            PAGE_GLASS => {
                palette.set_colors(color::GLASS_COLOR_TABLE.to_vec());
                palette.set_selected(profile.glass.color as usize);
            },
            PAGE_MUSTACHE => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.hair.color as usize);
            },
            PAGE_BEARD => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.hair.color as usize);
            },
            PAGE_MOLE => {
                palette.set_colors(vec![]);
            },
            _ => {
                palette.set_colors(vec![]);
            },
        }
    }

    fn on_ui_event(&mut self, events: Vec<ui::UIEvent>) {
        for event in events {
            match event.id {
                ID_STYLE_DEC => self.on_style_change(Delta::Dec),
                ID_STYLE_INC => self.on_style_change(Delta::Inc),
                ID_Y_DEC => self.on_y_change(Delta::Dec),
                ID_Y_INC => self.on_y_change(Delta::Inc),
                ID_X_DEC => self.on_x_change(Delta::Dec),
                ID_X_INC => self.on_x_change(Delta::Inc),
                ID_ROTATION_DEC => self.on_rotation_change(Delta::Dec),
                ID_ROTATION_INC => self.on_rotation_change(Delta::Inc),
                ID_SCALE_DEC => self.on_scale_change(Delta::Dec),
                ID_SCALE_INC => self.on_scale_change(Delta::Inc),
                ID_Y_SCALE_DEC => self.on_y_scale_change(Delta::Dec),
                ID_Y_SCALE_INC => self.on_y_scale_change(Delta::Inc),
                ID_PALETTE => self.on_color_change_from_palette(),
                _ => {
                    if event.id >= ID_PAGE_BUTTON_BEGIN && event.id < ID_PAGE_BUTTON_END {
                        self.on_page_change((event.id - ID_PAGE_BUTTON_BEGIN) as u8);
                    }
                }
            }


        }
    }

    fn run (&mut self, events_loop: &mut glutin::EventsLoop) {
        self.on_page_change(0);

        let mut rotate = 0.0;
        let mut running = true;
        let mut aspect = 1.0;
        let mut window_width = 0.0;
        let mut window_height = 0.0;
        while running {
            events_loop.poll_events(|event| {
                match event {
                    glutin::Event::WindowEvent{ event, .. } => match event {
                        glutin::WindowEvent::CursorEntered {..} => {
                            let events = self.scene.on_mouse_event(ui::MouseEvent::Entered, aspect);
                            self.on_ui_event(events);
                        },
                        glutin::WindowEvent::CursorLeft {..} => {
                            let events = self.scene.on_mouse_event(ui::MouseEvent::Left, aspect);
                            self.on_ui_event(events);
                        },
                        glutin::WindowEvent::MouseInput {state: glutin::ElementState::Pressed, ..} => {
                            let events = self.scene.on_mouse_event(ui::MouseEvent::Pressed, aspect);
                            self.on_ui_event(events);
                        },
                        glutin::WindowEvent::MouseInput {state: glutin::ElementState::Released, ..} => {
                            let events = self.scene.on_mouse_event(ui::MouseEvent::Released, aspect);
                            self.on_ui_event(events);
                        },
                        glutin::WindowEvent::CursorMoved {position: LogicalPosition{x, y}, ..} => {
                            let events = self.scene.on_mouse_event(ui::MouseEvent::Moved(
                                (x / window_width) as f32 * aspect,
                                (y / window_height) as f32
                            ), aspect);
                            self.on_ui_event(events);
                        },
                        glutin::WindowEvent::CloseRequested => running = false,
                        glutin::WindowEvent::Resized(logical_size) => {
                            window_width = logical_size.width;
                            window_height = logical_size.height;
                            let dpi_factor = self.gl_window.get_hidpi_factor();
                            let physical = logical_size.to_physical(dpi_factor);
                            let physical_u32: (u32, u32) = physical.clone().into();
                            aspect = physical_u32.0 as f32 / physical_u32.1 as f32;
                            unsafe {
                                gl::Viewport(0, 0, physical_u32.0 as GLsizei, physical_u32.1 as GLsizei);
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
                                        self.on_page_change(PAGE_END - 1);
                                    } else {
                                        self.on_page_change(self.page - 1);
                                    }
                                },

                                Some(glutin::VirtualKeyCode::Down) => {
                                    if self.page == PAGE_END - 1 {
                                        self.on_page_change(0);
                                    } else {
                                        self.on_page_change(self.page + 1);
                                    }
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
                gl::Enable(gl::DEPTH_TEST);
            }

            let info = self.database.owned[self.profile_index].to_render_info();

            let object_tran = cgmath::Matrix4::from_angle_y(cgmath::Deg(rotate));

            self.head_renderer.render_head(&info, &object_tran, aspect);

            unsafe {
                gl::Disable(gl::CULL_FACE);
                gl::Disable(gl::DEPTH_TEST);
            }

            self.scene.render(aspect);
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
        println!("Usage: citrii [Path to CFL_Res.dat] [Path to CFL_DB.dat]");
        return;
    }
    let mut events_loop = glutin::EventsLoop::new();
    let mut instance = Main::new(&args[1], &args[2], &mut events_loop);
    instance.run(&mut events_loop);
}
