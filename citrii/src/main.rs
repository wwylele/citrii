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
mod romfs;

use std::rc::Rc;
use std::cell::RefCell;

use glutin::dpi::*;
use glutin::GlContext;
use gl::types::*;

use byte_struct::*;


use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

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
const ID_FLIP_HAIR: u32 = 212;
const ID_PALETTE: u32 = 300;
const ID_EXTRA_FOLD: u32 = 400;
const ID_FAVORITE: u32 = 500;
const ID_SHARE: u32 = 501;
const ID_COPY: u32 = 502;
const ID_MALE: u32 = 503;
const ID_FEMALE: u32 = 504;
const ID_FAVORITE_COLOR: u32 = 505;
const ID_WIDTH: u32 = 506;
const ID_HEIGHT: u32 = 507;
const ID_NAME: u32 = 508;
const ID_AUTHOR: u32 = 509;
const ID_BIRTHMONTH_BEGIN: u32 = 600;
const ID_BIRTHDAY_BEGIN: u32 = 700;
const ID_SAVE: u32 = 800;
const ID_PROFILE_LIST_FOLD: u32 = 900;
const ID_PROFILE_LIST: u32 = 1000;
const ID_PROFILE_LEFT: u32 = 1100;
const ID_PROFILE_RIGHT: u32 = 1101;
const ID_PROFILE_UP: u32 = 1102;
const ID_PROFILE_DOWN: u32 = 1103;
const ID_PROFILE_DELETE: u32 = 1104;

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


const BIRTH_DAY_RANGE: [(u16, u16); 13] = [
    (0, 0),
    (1, 31),
    (1, 29),
    (1, 31),
    (1, 30),
    (1, 31),
    (1, 30),
    (1, 31),
    (1, 31),
    (1, 30),
    (1, 31),
    (1, 30),
    (1, 31),
];

fn clamp_birthday(month: u16, day: &mut u16) {
    let range = BIRTH_DAY_RANGE[month as usize];
    if *day < range.0 {
        *day = range.0;
    } else if *day > range.1 {
        *day = range.1;
    }
}

#[allow(clippy::if_same_then_else)]
fn adjust_birthmonth(day: u16, month: &mut u16) {
    if day == 0 {
        *month = 0;
    } else if *month == 0 {
        *month = 1;
    } else if day == 30 && *month == 2 {
        *month = 1;
    } else if day == 31 {
        if let 2|4|6|9|12 = *month {
            *month -= 1;
        }
    }
}


struct Main {
    clipboard_context: Option<ClipboardContext>,
    gl_window: glutin::GlWindow,

    database_filename: std::path::PathBuf,

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
    button_flip_hair: Rc<RefCell<ui::Button>>,

    edit_name: Rc<RefCell<ui::TextEdit>>,
    edit_author: Rc<RefCell<ui::TextEdit>>,
    scroll_width: Rc<RefCell<ui::ScrollBar>>,
    scroll_height: Rc<RefCell<ui::ScrollBar>>,
    check_favorite: Rc<RefCell<ui::CheckBox>>,
    check_share: Rc<RefCell<ui::CheckBox>>,
    check_copy: Rc<RefCell<ui::CheckBox>>,
    check_male: Rc<RefCell<ui::CheckBox>>,
    check_female: Rc<RefCell<ui::CheckBox>>,
    palette_favorite_color: Rc<RefCell<ui::Palette>>,
    buttons_birthmonth: Vec<Rc<RefCell<ui::Button>>>,
    buttons_birthday: Vec<Rc<RefCell<ui::Button>>>,
    layout_extra: Rc<RefCell<ui::GridLayout>>,
    buttons_profile_list: Vec<Rc<RefCell<ui::Button>>>,
    button_profile_delete: Rc<RefCell<ui::Button>>,
    layout_profile_list: Rc<RefCell<ui::GridLayout>>,
    icon_new_profile: Rc<texture::Texture>,
}

impl Main {
    fn new(asset_filename: std::path::PathBuf, database_filename: std::path::PathBuf, events_loop: &mut glutin::EventsLoop) -> Main {
        let clipboard_context =
            ClipboardProvider::new()
            .map_err(|e|println!("Failed to get clipboard: {}", e))
            .ok();

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

        let asset_romfs = std::fs::read(asset_filename).expect("Unable to read RomFS");
        let asset_data = romfs::get_romfs_file(&asset_romfs, &["CFL_Res.dat".to_string()]).expect("The provided RomFS is corrupted");;
        let asset = asset::Asset::from_bytes(&asset_data).expect("The provided CFL_Res.dat is corrupted");
        let head_renderer = head_renderer::HeadRenderer::with_asset(asset);

        let database_data = std::fs::read(&database_filename).expect("Unable to read CFL_Res.dat");
        let crc_a = crc::crc16_ninty(&database_data[0 .. 0xC81E]);
        assert_eq!(crc_a, u16::from_be_bytes([database_data[0xC81E], database_data[0xC81F]]));
        let crc_b = crc::crc16_ninty(&database_data[0xC820 .. 0xE4BE]);
        assert_eq!(crc_b, u16::from_be_bytes([database_data[0xE4BE], database_data[0xE4BF]]));
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
            page_buttons.iter().cloned().map(|m|->Rc<RefCell<dyn ui::UIElement>>{m}).collect(),
            0.02, 0.02, 0.02, 0.02, 0.01, 0.01, rect_renderer.clone());
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
            ui::ButtonContent::from_image(include_bytes!("icon/up.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_y_inc = ui::Button::new(ID_Y_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/down.png")),
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
            ui::ButtonContent::from_image(include_bytes!("icon/rotate-down.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let button_rotation_inc = ui::Button::new(ID_ROTATION_INC, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/rotate-up.png")),
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

        let button_flip_hair = ui::Button::new(ID_FLIP_HAIR, 0.1, 0.1,
            ui::ButtonContent::from_image(include_bytes!("icon/flip.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let layout_controls = ui::GridLayout::new(2, 8, vec![
            button_style_dec, button_style_inc,
            button_y_dec.clone(), button_y_inc.clone(),
            button_x_dec.clone(), button_x_inc.clone(),
            button_x_space_dec.clone(), button_x_space_inc.clone(),
            button_y_scale_inc.clone(), button_y_scale_dec.clone(),
            button_rotation_dec.clone(), button_rotation_inc.clone(),
            button_scale_inc.clone(), button_scale_dec.clone(),
            button_flip_hair.clone(), ui::Placeholder::new(),
        ], 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, rect_renderer.clone());

        let palette = ui::Palette::new(ID_PALETTE, 0.07, 1, rect_renderer.clone());

        let layout_controls_ex = ui::GridLayout::new(2, 1, vec![
            palette.clone(), layout_controls
        ], 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, rect_renderer.clone());

        let docker_controls = ui::Docker::new(layout_controls_ex, ui::XAlign::Right, ui::YAlign::Top);

        let label_name = ui::Label::new(0.2, 0.04, "Name", text_renderer.clone());
        let edit_name = ui::TextEdit::new(0.25, 0.04, rect_renderer.clone(), text_renderer.clone());
        let button_name = ui::Button::new(ID_NAME, 0.04, 0.04,
            ui::ButtonContent::from_image(include_bytes!("icon/paste.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let layout_name = ui::GridLayout::new(3, 1, vec![label_name, edit_name.clone(), button_name],
            0.0, 0.0, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_author = ui::Label::new(0.2, 0.04, "Author", text_renderer.clone());
        let edit_author = ui::TextEdit::new(0.25, 0.04, rect_renderer.clone(), text_renderer.clone());
        let button_author = ui::Button::new(ID_AUTHOR, 0.04, 0.04,
            ui::ButtonContent::from_image(include_bytes!("icon/paste.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let layout_author = ui::GridLayout::new(3, 1, vec![label_author, edit_author.clone(), button_author],
            0.0, 0.0, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_width = ui::Label::new(0.2, 0.04, "Width", text_renderer.clone());
        let scroll_width = ui::ScrollBar::new(ID_WIDTH, 0.28, 0.04, rect_renderer.clone());
        let layout_width = ui::GridLayout::new(2, 1, vec![label_width, scroll_width.clone()],
            0.0, 0.02, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_height = ui::Label::new(0.2, 0.04, "Height", text_renderer.clone());
        let scroll_height = ui::ScrollBar::new(ID_HEIGHT, 0.28, 0.04, rect_renderer.clone());
        let layout_height = ui::GridLayout::new(2, 1, vec![label_height, scroll_height.clone()],
            0.0, 0.02, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let image_unchecked = Rc::new(texture::Texture::from_png(include_bytes!("icon/uncheckbox.png")));
        let image_checked = Rc::new(texture::Texture::from_png(include_bytes!("icon/checkbox.png")));

        let label_favorite = ui::Label::new(0.3, 0.04, "Favorite", text_renderer.clone());
        let check_favorite = ui::CheckBox::new(ID_FAVORITE, 0.04, image_unchecked.clone(), image_checked.clone(), rect_renderer.clone());
        let layout_favorite = ui::GridLayout::new(2, 1, vec![label_favorite, check_favorite.clone()],
            0.0, 0.0, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_share = ui::Label::new(0.3, 0.04, "Allow share", text_renderer.clone());
        let check_share = ui::CheckBox::new(ID_SHARE, 0.04, image_unchecked.clone(), image_checked.clone(), rect_renderer.clone());
        let layout_share = ui::GridLayout::new(2, 1, vec![label_share, check_share.clone()],
            0.0, 0.0, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_copy = ui::Label::new(0.3, 0.04, "Allow copy", text_renderer.clone());
        let check_copy = ui::CheckBox::new(ID_COPY, 0.04, image_unchecked.clone(), image_checked.clone(), rect_renderer.clone());
        let layout_copy = ui::GridLayout::new(2, 1, vec![label_copy, check_copy.clone()],
            0.0, 0.0, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_gender = ui::Label::new(0.3, 0.04, "Male / Female", text_renderer.clone());
        let check_male = ui::CheckBox::new(ID_MALE, 0.04, image_unchecked.clone(), image_checked.clone(), rect_renderer.clone());
        let check_female = ui::CheckBox::new(ID_FEMALE, 0.04, image_unchecked.clone(), image_checked.clone(), rect_renderer.clone());
        let layout_gender = ui::GridLayout::new(3, 1,
            vec![check_male.clone(), label_gender, check_female.clone()],
            0.0, 0.0, 0.0, 0.0, 0.01, 0.0, rect_renderer.clone());

        let label_favorite_color = ui::Label::new(0.4, 0.04, "Favorite color", text_renderer.clone());
        let palette_favorite_color = ui::Palette::new(ID_FAVORITE_COLOR, 0.07, 6, rect_renderer.clone());
        palette_favorite_color.borrow_mut().set_colors(color::WEARING_COLOR_TABLE.to_vec());
        let layout_favorite_color = ui::GridLayout::new(1, 2,
            vec![label_favorite_color, palette_favorite_color.clone()],
            0.0, 0.0, 0.02, 0.0, 0.0, 0.01, rect_renderer.clone());

        let buttons_birthmonth: Vec<Rc<RefCell<ui::Button>>> = (0..13).map(|i|
                {ui::Button::new(ID_BIRTHMONTH_BEGIN + i, 0.03, 0.03, ui::ButtonContent::Text(i.to_string()),
                 rect_renderer.clone(), text_renderer.clone())}
            ).collect();

        let buttons_birthday: Vec<Rc<RefCell<ui::Button>>> = (0..32).map(|i|
                {ui::Button::new(ID_BIRTHDAY_BEGIN + i, 0.03, 0.03, ui::ButtonContent::Text(i.to_string()),
                 rect_renderer.clone(), text_renderer.clone())}
            ).collect();

        let placeholder: Rc<RefCell<dyn ui::UIElement>> = ui::Placeholder::new();

        let layout_birthmonth = ui::GridLayout::new(4, 4,
            buttons_birthmonth.iter().cloned().map(|m|->Rc<RefCell<dyn ui::UIElement>>{m})
            .chain(std::iter::repeat(placeholder).take(3)).collect(),
            0.0, 0.0, 0.0, 0.0, 0.005, 0.005, rect_renderer.clone());
        let layout_birthday = ui::GridLayout::new(8, 4,
            buttons_birthday.iter().cloned().map(|m|->Rc<RefCell<dyn ui::UIElement>>{m}).collect(),
            0.0, 0.0, 0.0, 0.0, 0.005, 0.005, rect_renderer.clone());

        let label_birthday_main = ui::Label::new(0.4, 0.04, "Birthday", text_renderer.clone());
        let label_birthmonth = ui::Label::new(0.15, 0.03, "Month", text_renderer.clone());
        let label_birthday = ui::Label::new(0.2, 0.03, "Day", text_renderer.clone());

        let layout_birthday_meta = ui::GridLayout::new(2, 2,
            vec![label_birthmonth, label_birthday, layout_birthmonth, layout_birthday],
            0.0, 0.0, 0.0, 0.0, 0.02, 0.02, rect_renderer.clone());

        let layout_extra = ui::GridLayout::new(1, 11, vec![
            layout_name,
            layout_author,
            layout_height,
            layout_width,
            layout_favorite,
            layout_share,
            layout_copy,
            layout_gender,
            layout_favorite_color,
            label_birthday_main,
            layout_birthday_meta,
            ], 0.0, 0.0, 0.02, 0.02, 0.01, 0.005, rect_renderer.clone());
        layout_extra.borrow_mut().set_color((0.8, 0.8, 1.0, 0.8));

        let button_extra = ui::Button::new(ID_EXTRA_FOLD, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/edit.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let layout_extra_ex = ui::GridLayout::new(1, 2, vec![layout_extra.clone(), button_extra],
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, rect_renderer.clone());
        layout_extra.borrow_mut().set_visible(false);
        let docker_extra = ui::Docker::new(layout_extra_ex, ui::XAlign::Center, ui::YAlign::Top);

        let button_save = ui::Button::new(ID_SAVE, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/save.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let docker_save = ui::Docker::new(button_save, ui::XAlign::Left, ui::YAlign::Bottom);

        let buttons_profile_list: Vec<Rc<RefCell<ui::Button>>> = (0..100).map(|i|
                {ui::Button::new(ID_PROFILE_LIST + i, 0.05, 0.05, ui::ButtonContent::Text(i.to_string()),
                 rect_renderer.clone(), text_renderer.clone())}
            ).collect();

        let layout_profile_list_inner = ui::GridLayout::new(10, 10,
            buttons_profile_list.iter().cloned().map(|m|->Rc<RefCell<dyn ui::UIElement>>{m}).collect(),
            0.01, 0.01, 0.01, 0.01, 0.005, 0.005, rect_renderer.clone());

        let button_profile_left = ui::Button::new(ID_PROFILE_LEFT, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/moveleft.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let button_profile_right = ui::Button::new(ID_PROFILE_RIGHT, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/moveright.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let button_profile_up = ui::Button::new(ID_PROFILE_UP, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/moveup.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let button_profile_down = ui::Button::new(ID_PROFILE_DOWN, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/movedown.png")),
            rect_renderer.clone(),
            text_renderer.clone());
        let button_profile_delete = ui::Button::new(ID_PROFILE_DELETE, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/delete.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let layout_profile_list_control = ui::GridLayout::new(5, 1,
            vec![
                button_profile_left,
                button_profile_right,
                button_profile_up,
                button_profile_down,
                button_profile_delete.clone(),
            ], 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, rect_renderer.clone());

        let layout_profile_list = ui::GridLayout::new(1, 2,
            vec![layout_profile_list_inner, layout_profile_list_control],
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, rect_renderer.clone());

        layout_profile_list.borrow_mut().set_visible(false);
        layout_profile_list.borrow_mut().set_color((0.8, 0.8, 1.0, 0.8));

        let button_profile_list_fold = ui::Button::new(ID_PROFILE_LIST_FOLD, 0.07, 0.07,
            ui::ButtonContent::from_image(include_bytes!("icon/people.png")),
            rect_renderer.clone(),
            text_renderer.clone());

        let layout_profile_list_ex = ui::GridLayout::new(1, 2,
            vec![button_profile_list_fold, layout_profile_list.clone()],
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, rect_renderer.clone());

        let docker_profile_list = ui::Docker::new(layout_profile_list_ex, ui::XAlign::Center, ui::YAlign::Bottom);

        let scene = ui::Scene::new(vec![docker_pages, docker_controls, docker_extra, docker_save, docker_profile_list]);

        let icon_new_profile = Rc::new(texture::Texture::from_png(include_bytes!("icon/new.png")));

        Main {
            clipboard_context,
            gl_window,
            database_filename,
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
            button_flip_hair,
            edit_name,
            edit_author,
            scroll_width,
            scroll_height,
            check_favorite,
            check_share,
            check_copy,
            check_male,
            check_female,
            palette_favorite_color,
            buttons_birthmonth,
            buttons_birthday,
            layout_extra,
            buttons_profile_list,
            button_profile_delete,
            layout_profile_list,
            icon_new_profile,
        }
    }

    fn update_profile_list(&self) {
        let mut slot_map: [Option<usize>; 100] = [None; 100];
        for (i, profile) in self.database.owned.iter().enumerate() {
            if profile.main.is_null() {
                continue;
            }
            let slot = profile.main.get_slot();
            assert!(slot_map[slot] == None);
            slot_map[slot] = Some(i);
        }

        for (j, slot) in slot_map.iter().enumerate() {
            let mut button = self.buttons_profile_list[j].borrow_mut();
            button.set_content(match slot {
                None => ui::ButtonContent::from_text(""),
                Some(i) => {
                    ui::ButtonContent::from_text(&String::from_utf16_lossy(
                        &[self.database.owned[*i].main.name[0]]))
                }
            });

            button.set_selected(match slot {
                None => false,
                Some(i) => *i == self.profile_index
            });

            button.set_hover_image(match slot {
                None => Some(self.icon_new_profile.clone()),
                Some(_) => None,
            })
        }

        self.button_profile_delete.borrow_mut().set_visible(self.profile_index != 0);
    }

    fn update_profile_extra(&self) {
        fn name_to_text(name: &[u16]) -> String {
            let end = name.iter().position(|&v| v == 0).unwrap_or_else(||name.len());
            String::from_utf16_lossy(&name[0 .. end])
        }

        let profile_ex = &self.database.owned[self.profile_index];
        let profile = &profile_ex.main;
        self.edit_name.borrow_mut().set_text(name_to_text(&profile.name[..]));
        self.edit_author.borrow_mut().set_text(name_to_text(&profile_ex.author[..]));

        for (i, button) in self.buttons_birthmonth.iter().enumerate() {
            button.borrow_mut().set_selected(profile.general.birth_month as usize == i);
        }

        for (i, button) in self.buttons_birthday.iter().enumerate() {
            button.borrow_mut().set_selected(profile.general.birth_day as usize == i);
        }

        self.scroll_width.borrow_mut().set_value(profile.width as f32 / 127.0);
        self.scroll_height.borrow_mut().set_value(profile.height as f32 / 127.0);
        self.check_favorite.borrow_mut().set_checked(profile.general.favorite != 0);
        self.check_share.borrow_mut().set_checked(profile.face.disable_sharing == 0);
        self.check_copy.borrow_mut().set_checked(profile.header.allow_copying != 0);
        self.check_male.borrow_mut().set_checked(profile.general.sex == 0);
        self.check_female.borrow_mut().set_checked(profile.general.sex != 0);
        self.palette_favorite_color.borrow_mut().set_selected(profile.general.favorite_color as usize);
    }

    fn on_style_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index].main;
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
        let profile = &mut self.database.owned[self.profile_index].main;
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
        let profile = &mut self.database.owned[self.profile_index].main;
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
        let profile = &mut self.database.owned[self.profile_index].main;
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
        let profile = &mut self.database.owned[self.profile_index].main;
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.y_scale, 0, 6, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.y_scale, 0, 6, delta),
            PAGE_LIP => clamp_change_value(&mut profile.lip.y_scale, 0, 6, delta),
            _ => (),
        }
    }

    fn on_rotation_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index].main;
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.rotation, 0, 11, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.rotation, 0, 7, delta),
            _ => (),
        }
    }

    fn on_x_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index].main;
        match self.page {
            PAGE_EYEBROW => clamp_change_value(&mut profile.eyebrow.x, 0, 12, delta),
            PAGE_EYE => clamp_change_value(&mut profile.eye.x, 0, 12, delta),
            PAGE_MOLE => clamp_change_value(&mut profile.mole.x, 0, 0x10, delta),
            _ => (),
        }
    }

    fn on_y_change(&mut self, delta: Delta) {
        let profile = &mut self.database.owned[self.profile_index].main;
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
        let mut database_data = vec![0u8; database::Database::BYTE_LEN];
        self.database.write_bytes(&mut database_data[..]);
        let crc_a = crc::crc16_ninty(&database_data[0 .. 0xC81E]).to_be_bytes();
        database_data[0xC81E .. 0xC820].copy_from_slice(&crc_a);
        let crc_b = crc::crc16_ninty(&database_data[0xC820 .. 0xE4BE]).to_be_bytes();
        database_data[0xE4BE .. 0xE4C0].copy_from_slice(&crc_b);
        match std::fs::write(&self.database_filename, &database_data) {
            Ok(()) => println!("Saved"),
            Err(e) => println!("Failed to save: {}", e)
        };
    }

    fn get_string_from_clipboard(&mut self) -> Option<[u16; 10]> {
        if let Some(c) = &mut self.clipboard_context {
            match c.get_contents() {
                Ok(s) => {
                    let v: Vec<u16> = s.encode_utf16().chain(std::iter::repeat(0)).take(10).collect();
                    Some([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9]])
                },
                Err(e) => {
                    println!("Clipboard error: {}", e);
                    None
                }
            }
        } else {
            None
        }
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
        self.button_flip_hair.borrow_mut().set_visible(page == PAGE_HAIR);

        let profile = &self.database.owned[self.profile_index];
        let mut palette = self.palette.borrow_mut();
        match page {
            PAGE_FACE => {
                palette.set_colors(color::SKIN_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.face.color as usize);
            },
            PAGE_MAKEUP => {
                palette.set_colors(vec![]);
            },
            PAGE_WRINKLE => {
                palette.set_colors(vec![]);
            },
            PAGE_HAIR => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.hair.color as usize);
            },
            PAGE_EYEBROW => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.eyebrow.color as usize);
            },
            PAGE_EYE => {
                palette.set_colors(color::EYE_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.eye.color as usize);
            },
            PAGE_NOSE => {
                palette.set_colors(vec![]);
            },
            PAGE_LIP => {
                palette.set_colors(color::LIP_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.lip.color as usize);
            },
            PAGE_GLASS => {
                palette.set_colors(color::GLASS_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.glass.color as usize);
            },
            PAGE_MUSTACHE => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.hair.color as usize);
            },
            PAGE_BEARD => {
                palette.set_colors(color::HAIR_COLOR_TABLE.to_vec());
                palette.set_selected(profile.main.hair.color as usize);
            },
            PAGE_MOLE => {
                palette.set_colors(vec![]);
            },
            _ => {
                palette.set_colors(vec![]);
            },
        }
    }

    fn on_slot_move(&mut self, modifier: fn(usize) -> Option<usize>) {
        let slot = self.database.owned[self.profile_index].main.get_slot();
        if let Some(other_slot) = modifier(slot) {
            let other = self.database.owned_slot_to_index(other_slot);
            self.database.owned[self.profile_index].main.set_slot(other_slot);
            if let Some(i) = other {
                self.database.owned[i].main.set_slot(slot);
            }
            self.update_profile_list();
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
                ID_FLIP_HAIR => {
                    let flip = &mut self.database.owned[self.profile_index].main.hair.flip;
                    *flip = 1 - *flip;
                }
                ID_PALETTE => self.on_color_change_from_palette(),
                ID_EXTRA_FOLD => {
                    let visible = self.layout_extra.borrow().get_visible();
                    self.layout_extra.borrow_mut().set_visible(!visible);
                    if !visible {
                        self.layout_profile_list.borrow_mut().set_visible(false);
                    }
                }
                ID_PROFILE_LIST_FOLD => {
                    let visible = self.layout_profile_list.borrow().get_visible();
                    self.layout_profile_list.borrow_mut().set_visible(!visible);
                    if !visible {
                        self.layout_extra.borrow_mut().set_visible(false);
                    }
                }
                ID_FAVORITE => {
                    let b = &mut self.database.owned[self.profile_index].main.general.favorite;
                    *b = 1 - *b;
                    self.update_profile_extra();
                },
                ID_SHARE => {
                    let b = &mut self.database.owned[self.profile_index].main.face.disable_sharing;
                    *b = 1 - *b;
                    self.update_profile_extra();
                },
                ID_COPY => {
                    let b = &mut self.database.owned[self.profile_index].main.header.allow_copying;
                    *b = 1 - *b;
                    self.update_profile_extra();
                },
                ID_MALE => {
                    self.database.owned[self.profile_index].main.general.sex = 0;
                    self.update_profile_extra();
                },
                ID_FEMALE => {
                    self.database.owned[self.profile_index].main.general.sex = 1;
                    self.update_profile_extra();
                },
                ID_FAVORITE_COLOR => {
                    self.database.owned[self.profile_index].main.general.favorite_color =
                        self.palette_favorite_color.borrow().get_selected() as u16;
                    self.update_profile_extra();
                },
                ID_WIDTH => {
                    self.database.owned[self.profile_index].main.width =
                        (self.scroll_width.borrow().get_value() * 127.0).round() as u8;
                    self.update_profile_extra();
                },
                ID_HEIGHT => {
                    self.database.owned[self.profile_index].main.height =
                        (self.scroll_height.borrow().get_value() * 127.0).round() as u8;
                    self.update_profile_extra();
                },
                ID_NAME => {
                    if let Some(s) = self.get_string_from_clipboard() {
                        self.database.owned[self.profile_index].main.name = s;
                        self.update_profile_extra();
                        self.update_profile_list();
                    }
                }
                ID_AUTHOR => {
                    if let Some(s) = self.get_string_from_clipboard() {
                        self.database.owned[self.profile_index].author = s;
                        self.update_profile_extra();
                    }
                }
                ID_SAVE => {
                    self.on_save();
                }
                ID_PROFILE_LEFT => {
                    fn mover(slot: usize) -> Option<usize> {
                        if slot % 10 == 0 { None } else { Some(slot - 1) }
                    }
                    self.on_slot_move(mover);
                }
                ID_PROFILE_RIGHT => {
                    fn mover(slot: usize) -> Option<usize> {
                        if slot % 10 == 9 { None } else { Some(slot + 1) }
                    }
                    self.on_slot_move(mover);
                }
                ID_PROFILE_UP => {
                    fn mover(slot: usize) -> Option<usize> {
                        if slot / 10 == 0 { None } else { Some(slot - 10) }
                    }
                    self.on_slot_move(mover);
                }
                ID_PROFILE_DOWN => {
                    fn mover(slot: usize) -> Option<usize> {
                        if slot / 10 == 9 { None } else { Some(slot + 10) }
                    }
                    self.on_slot_move(mover);
                }
                ID_PROFILE_DELETE => {
                    self.database.owned[self.profile_index] = database::ProfileFull::default();
                    self.profile_index = 0;
                    self.update_profile_extra();
                    self.update_profile_list();
                }

                _ => {
                    if event.id >= ID_PAGE_BUTTON_BEGIN && event.id < ID_PAGE_BUTTON_END {
                        self.on_page_change((event.id - ID_PAGE_BUTTON_BEGIN) as u8);
                    } else if event.id >= ID_BIRTHMONTH_BEGIN && event.id < ID_BIRTHMONTH_BEGIN + 50 {
                        let birthmonth = (event.id - ID_BIRTHMONTH_BEGIN) as u16;
                        let profile = &mut self.database.owned[self.profile_index].main.general;
                        profile.birth_month = birthmonth;
                        clamp_birthday(birthmonth, &mut profile.birth_day);
                        self.update_profile_extra();
                    } else if event.id >= ID_BIRTHDAY_BEGIN && event.id < ID_BIRTHDAY_BEGIN + 50 {
                        let profile = &mut self.database.owned[self.profile_index].main.general;
                        let birthday = (event.id - ID_BIRTHDAY_BEGIN) as u16;
                        profile.birth_day = birthday;
                        adjust_birthmonth(birthday, &mut profile.birth_month);
                        self.update_profile_extra();
                    } else if event.id >= ID_PROFILE_LIST && event.id < ID_PROFILE_LIST + 100 {
                        let slot = (event.id - ID_PROFILE_LIST) as usize;
                        if let Some(i) = self.database.owned_slot_to_index(slot) {
                            self.profile_index = i;
                            self.update_profile_extra();
                            self.update_profile_list();
                        } else {
                            let index = self.database.owned.iter().position(|profile|profile.main.is_null()).unwrap();
                            let mac = self.database.owned[0].main.id.mac.clone();
                            let system_id = self.database.owned[0].main.system_id.clone();
                            let now = chrono::Local::now().naive_local();
                            self.database.owned[index].main = database::Profile::new(
                                mac, system_id, now, slot
                            );
                            self.profile_index = index;
                            self.update_profile_extra();
                            self.update_profile_list();
                        }
                    }
                }
            }
        }
    }

    fn run (&mut self, events_loop: &mut glutin::EventsLoop) {
        self.on_page_change(0);
        self.update_profile_extra();
        self.update_profile_list();

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
                            let physical_u32: (u32, u32) = physical.into();
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

            let info = self.database.owned[self.profile_index].main.to_render_info();

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
    if args.len() < 2 {
        println!("Usage: citrii [Path citra user folder]");
        return;
    }
    let citra_path = std::path::Path::new(&args[1]);
    let asset_path = citra_path.join("nand/00000000000000000000000000000000/title/0004009b/00010202/content/00000000.app.romfs");
    let database_path = citra_path.join("nand/data/00000000000000000000000000000000/extdata/00048000/F000000B/user/CFL_DB.dat");
    let mut events_loop = glutin::EventsLoop::new();
    let mut instance = Main::new(asset_path, database_path, &mut events_loop);
    instance.run(&mut events_loop);
}
