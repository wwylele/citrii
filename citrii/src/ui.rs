// UI coordinates:
// (0, 0) is at top-left
// (w, 1) is at bottom-right
// w = window_width / window_height

// GL coordinates (name prefixed with gl_):
// (-1, -1) is at bottom-left
// (1, 1) is at top-right
// aspect = window_width / window_height

use crate::rect_renderer;
use crate::text_renderer;
use crate::texture;
use crate::color;

use std::rc::Rc;
use std::cell::RefCell;

pub enum MouseEvent {
    Entered,
    Left,
    Moved(f32, f32),
    Pressed,
    Released,
}

pub struct UIEvent {
    pub id: u32
}

pub trait UIElement {
    fn get_size(&self) -> /* width and height in UI dimension unit */ (f32, f32);
    fn render(&self, gl_bottom_left: (f32, f32), gl_top_right: (f32, f32));
    fn on_mouse_event(&mut self, _event: MouseEvent) -> Vec<UIEvent> {
        vec![]
    }
}

pub enum ButtonContent {
    Text(String),
    Image(texture::Texture),
}

impl ButtonContent {
    pub fn from_text(text: &str) -> ButtonContent {
        ButtonContent::Text(String::from(text))
    }

    pub fn from_image(data: &[u8]) -> ButtonContent {
        if let Ok(image::DynamicImage::ImageRgba8(image_buffer)) = image::load_from_memory(data) {
            let width = image_buffer.width() as usize;
            let height = image_buffer.height() as usize;
            ButtonContent::Image(texture::Texture::new(width, height, &image_buffer.into_raw(),
                &texture::WrapMode::Edge, &texture::WrapMode::Edge))
        } else {
            panic!("broken font image");
        }


    }
}

pub struct Button {
    id: u32,
    w: f32,
    h: f32,
    content: ButtonContent,
    rect_renderer: Rc<rect_renderer::RectRenderer>,
    text_renderer: Rc<text_renderer::TextRenderer>,

    cursor_in: bool,
    selected: bool,
    visible: bool,
}

impl Button {
    pub fn new(id: u32, w: f32, h: f32, content: ButtonContent,
        rect_renderer: Rc<rect_renderer::RectRenderer>,
        text_renderer: Rc<text_renderer::TextRenderer>) -> Rc<RefCell<Button>> {

        Rc::new(RefCell::new(Button {
            id, w, h, content, rect_renderer, text_renderer,
            cursor_in: false,
            selected: false,
            visible: true,
        }))
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl UIElement for Button {
    fn get_size(&self) -> (f32, f32) {
        (self.w, self.h)
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        if !self.visible {return}
        let aspect = (self.w / (gl_x1 - gl_x0)) / (self.h / (gl_y1 - gl_y0));
        self.rect_renderer.render(((gl_x0, gl_y0), (gl_x1, gl_y1)),
            if self.selected
                {rect_renderer::Filling::Color(1.0, 0.5, 0.5, 0.3)}
            else if self.cursor_in
                {rect_renderer::Filling::Color(1.0, 1.0, 0.5, 0.3)}
            else
                {rect_renderer::Filling::Color(1.0, 1.0, 1.0, 0.3)}
            );

        match self.content {
            ButtonContent::Text(ref text) => {
                self.text_renderer.render(&text, ((gl_x0 + gl_x1) * 0.5, (gl_y0 + gl_y1) * 0.5),
                    gl_y1 - gl_y0, (0.0, 0.0, 0.0), aspect);
            },
            ButtonContent::Image(ref image) => {
                self.rect_renderer.render(((gl_x0, gl_y0), (gl_x1, gl_y1)),
                    rect_renderer::Filling::Texture(&image, ((0.0, 0.0), (1.0, 1.0)), (1.0, 1.0, 1.0)));
            },
        }

    }
    fn on_mouse_event(&mut self, event: MouseEvent) -> Vec<UIEvent> {
        match event {
            MouseEvent::Entered => {
                self.cursor_in = true;
            },
            MouseEvent::Left => {
                self.cursor_in = false;
            }
            MouseEvent::Pressed => {
                return vec![UIEvent{id: self.id}]
            }
            _ => ()
        }
        vec![]
    }
}



pub struct Palette {
    id: u32,
    width: f32,
    selected: usize,
    cursor_in: Option<usize>,
    colors: Vec<(u8, u8, u8)>,
    square_texture: texture::Texture,
    rect_renderer: Rc<rect_renderer::RectRenderer>,
}

impl Palette {
    pub fn new(id: u32, width: f32, rect_renderer: Rc<rect_renderer::RectRenderer>) -> Rc<RefCell<Palette>> {
        let texture = if let Ok(image::DynamicImage::ImageRgba8(image_buffer)) =
            image::load_from_memory(include_bytes!("icon/white-square.png")) {
            let width = image_buffer.width() as usize;
            let height = image_buffer.height() as usize;
            texture::Texture::new(width, height, &image_buffer.into_raw(),
                &texture::WrapMode::Edge, &texture::WrapMode::Edge)
        } else {
            panic!("broken font image");
        };

        Rc::new(RefCell::new(Palette {
            id,
            width,
            selected: 0,
            cursor_in: None,
            colors: vec![],
            square_texture: texture,
            rect_renderer
        }))
    }

    pub fn set_colors(&mut self, colors: Vec<(u8, u8, u8)>) {
        self.colors = colors;
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected;
    }

    pub fn get_selected(&self) -> usize {
        self.selected
    }
}

impl UIElement for Palette {
    fn get_size(&self) -> (f32, f32) {
        (self.width, self.width * self.colors.len() as f32)
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        let height = (gl_y1 - gl_y0) / self.colors.len() as f32;
        for i in 0 .. self.colors.len() {
            let y0 = gl_y0 + height * (self.colors.len() - i - 1) as f32;
            let y1 = y0 + height;

            self.rect_renderer.render(((gl_x0, y0), (gl_x1, y1)),
            if self.selected == i
                {rect_renderer::Filling::Color(1.0, 0.5, 0.5, 0.3)}
            else if self.cursor_in == Some(i)
                {rect_renderer::Filling::Color(1.0, 1.0, 0.5, 0.3)}
            else
                {rect_renderer::Filling::Color(1.0, 1.0, 1.0, 0.3)}
            );

            self.rect_renderer.render(((gl_x0, y0), (gl_x1, y1)),
                rect_renderer::Filling::Texture(&self.square_texture, ((0.0, 0.0), (1.0, 1.0)),
                color::convert_color(&self.colors[i])))

        }

    }
    fn on_mouse_event(&mut self, event: MouseEvent) -> Vec<UIEvent> {
        match event {
            MouseEvent::Left => {
                self.cursor_in = None;
            }
            MouseEvent::Pressed => {
                if let Some(selected) = self.cursor_in {
                    self.selected = selected;
                }
                return vec![UIEvent{id: self.id}]
            }
            MouseEvent::Moved(_x, y) => {
                let mut i = y / self.width;
                if i < 0.0 { i = 0.0; }
                let max = (self.colors.len() - 1) as f32;
                if i > max { i = max; }
                self.cursor_in = Some(i as usize);
            }
            _ => ()
        }
        vec![]
    }
}

pub struct GridLayout {
    x_count: usize,
    y_count: usize,
    children: Vec<Rc<RefCell<dyn UIElement>>>,
    xl_margin: f32,
    xr_margin: f32,
    yt_margin: f32,
    yb_margin: f32,
    x_gap: f32,
    y_gap: f32,
    cursor_in: Option<usize>,
}

impl GridLayout {
    pub fn new(x_count: usize, y_count: usize, children: Vec<Rc<RefCell<dyn UIElement>>>,
        xl_margin: f32, xr_margin: f32, yt_margin: f32, yb_margin: f32, x_gap: f32, y_gap: f32)
        -> Rc<RefCell<GridLayout>> {
        assert_eq!(x_count * y_count, children.len());
        Rc::new(RefCell::new(GridLayout {
            x_count,
            y_count,
            children,
            xl_margin,
            xr_margin,
            yt_margin,
            yb_margin,
            x_gap,
            y_gap,
            cursor_in: None,
        }))
    }

    fn get_grid_size(&self) -> (Vec<f32>, Vec<f32>) {
        let mut ws = vec![0.0; self.x_count];
        let mut hs = vec![0.0; self.y_count];
        for y in 0 .. self.y_count {
            for x in 0 .. self.x_count {
                let (w, h) = self.children[x + y * self.x_count].borrow().get_size();
                if w > ws[x] {ws[x] = w;}
                if h > hs[y] {hs[y] = h;}
            }
        }
        (ws, hs)
    }
}

impl UIElement for GridLayout {
    fn get_size(&self) -> (f32, f32) {
        let w_base = self.xl_margin + self.xr_margin + self.x_gap * (self.x_count - 1) as f32;
        let h_base = self.yt_margin + self.yb_margin + self.y_gap * (self.y_count - 1) as f32;
        let (ws, hs) = self.get_grid_size();

        (w_base + ws.iter().fold(0.0, |acc, a|acc + a), h_base + hs.iter().fold(0.0, |acc, a|acc + a))
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        // TODO: draw self

        let (w, h) = self.get_size();
        let w_ui_to_gl = (gl_x1 - gl_x0) / w;
        let h_ui_to_gl = (gl_y1 - gl_y0) / h;
        let (ws, hs) = self.get_grid_size();

        let mut cur_y = gl_y1 - self.yt_margin * h_ui_to_gl;
        for y in 0 .. self.y_count {
            let mut cur_x = gl_x0 + self.xl_margin * w_ui_to_gl;
            for x in 0 .. self.x_count {
                let child = self.children[x + y * self.x_count].borrow();
                let (cw, ch) = child.get_size();
                let x_begin = cur_x + (ws[x] - cw) * 0.5 * w_ui_to_gl;
                let y_begin = cur_y - (hs[y] - ch) * 0.5 * h_ui_to_gl;
                child.render((x_begin, y_begin - ch * h_ui_to_gl), (x_begin + cw * w_ui_to_gl, y_begin));
                cur_x += (ws[x] + self.x_gap) * w_ui_to_gl;
            }
            cur_y -= (hs[y] + self.y_gap) * h_ui_to_gl;
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent) -> Vec<UIEvent> {
        let mut ui_event = vec![];
        match event {
            MouseEvent::Entered => (),
            MouseEvent::Left => {
                if let Some(previous) = self.cursor_in {
                    ui_event.append(&mut self.children[previous].borrow_mut().on_mouse_event(event));
                }
                self.cursor_in = None;
            },
            MouseEvent::Moved(cursor_x, cursor_y) => {
                let mut current = None;
                let mut dx = 0.0;
                let mut dy = 0.0;

                let (ws, hs) = self.get_grid_size();
                let mut cur_y = self.yt_margin;
                for y in 0 .. self.y_count {
                    let mut cur_x = self.xl_margin;
                    for x in 0 .. self.x_count {
                        let i = x + y * self.x_count;
                        let child = self.children[x + y * self.x_count].borrow();
                        let (cw, ch) = child.get_size();
                        let x0 = cur_x + (ws[x] - cw) * 0.5;
                        let y0 = cur_y + (hs[y] - ch) * 0.5;
                        let x1 = x0 + cw;
                        let y1 = y0 + ch;
                        if cursor_x >= x0 && cursor_x <= x1 && cursor_y >= y0 && cursor_y <= y1 {
                            current = Some(i);
                            dx = cursor_x - x0;
                            dy = cursor_y - y0;
                            break;
                        }
                        cur_x += ws[x] + self.x_gap;
                    }
                    cur_y += hs[y] + self.y_gap;
                }

                if self.cursor_in != current {
                    if let Some(previous) = self.cursor_in {
                        ui_event.append(&mut self.children[previous].borrow_mut().on_mouse_event(MouseEvent::Left));
                    }
                    if let Some(current) = current {
                        ui_event.append(&mut self.children[current].borrow_mut().on_mouse_event(MouseEvent::Entered));
                    }
                    self.cursor_in = current;
                }
                if let Some(current) = current {
                    self.children[current].borrow_mut().on_mouse_event(MouseEvent::Moved(dx, dy));
                }
            },
            MouseEvent::Pressed | MouseEvent::Released => {
                if let Some(previous) = self.cursor_in {
                    ui_event.append(&mut self.children[previous].borrow_mut().on_mouse_event(event));
                }
            },
        }
        ui_event
    }
}

pub enum XAlign {
    Left,
    Center,
    Right,
}

pub enum YAlign {
    Top,
    Center,
    Bottom,
}

pub struct Docker {
    element: Rc<RefCell<dyn UIElement>>,
    x_align: XAlign,
    y_align: YAlign,
}

impl Docker {
    pub fn new(element: Rc<RefCell<dyn UIElement>>, x_align: XAlign, y_align: YAlign) -> Docker {
        Docker {
            element,
            x_align,
            y_align,
        }
    }

    pub fn get_ui_rect(&self, aspect: f32) -> ((f32, f32), (f32, f32)) {
        let child = self.element.borrow();
        let (w, h) = child.get_size();
        let x_begin = match self.x_align {
            XAlign::Left => 0.0,
            XAlign::Center => (aspect - w) * 0.5,
            XAlign::Right => aspect - w,
        };
        let y_begin = match self.y_align {
            YAlign::Top => 0.0,
            YAlign::Center => (1.0 - h) * 0.5,
            YAlign::Bottom => 1.0 - h,
        };

        ((x_begin, y_begin), (x_begin + w, y_begin + h))
    }

    pub fn render(&self, aspect: f32) {
        let ((x0, y0), (x1, y1)) = self.get_ui_rect(aspect);
        let x_begin = x0 / aspect * 2.0 - 1.0;
        let y_begin = (1.0 - y0) * 2.0 - 1.0;
        self.element.borrow().render(
            (x_begin, y_begin - (y1 - y0) * 2.0), (x_begin + (x1 - x0) * 2.0 / aspect, y_begin));
    }

    pub fn on_mouse_event(&mut self, event: MouseEvent) -> Vec<UIEvent> {
        self.element.borrow_mut().on_mouse_event(event)
    }
}

pub struct Scene {
    dockers: Vec<Docker>,
    cursor_in: Option<usize>,
}

impl Scene {
    pub fn new(dockers: Vec<Docker>) -> Scene {
        Scene{dockers, cursor_in: None}
    }
    pub fn on_mouse_event(&mut self, event: MouseEvent, aspect: f32) -> Vec<UIEvent> {
        let mut ui_event = vec![];
        match event {
            MouseEvent::Entered => (),
            MouseEvent::Left => {
                if let Some(previous) = self.cursor_in {
                    ui_event.append(&mut self.dockers[previous].on_mouse_event(event));
                }
                self.cursor_in = None;
            },
            MouseEvent::Moved(x, y) => {
                let mut current = None;
                let mut dx = 0.0;
                let mut dy = 0.0;
                for i in 0 .. self.dockers.len() {
                    let ((x0, y0), (x1, y1)) = self.dockers[i].get_ui_rect(aspect);
                    if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                        current = Some(i);
                        dx = x - x0;
                        dy = y - y0;
                        break;
                    }
                }
                if self.cursor_in != current {
                    if let Some(previous) = self.cursor_in {
                        ui_event.append(&mut self.dockers[previous].on_mouse_event(MouseEvent::Left));
                    }
                    if let Some(current) = current {
                        ui_event.append(&mut self.dockers[current].on_mouse_event(MouseEvent::Entered));
                    }
                    self.cursor_in = current;
                }
                if let Some(current) = current {
                    ui_event.append(&mut self.dockers[current].on_mouse_event(MouseEvent::Moved(dx, dy)));
                }
            },
            MouseEvent::Pressed | MouseEvent::Released => {
                if let Some(previous) = self.cursor_in {
                    ui_event.append(&mut self.dockers[previous].on_mouse_event(event));
                }
            },
        }
        ui_event
    }

    pub fn render(&self, aspect: f32) {
        for docker in self.dockers.iter() {
            docker.render(aspect);
        }
    }
}
