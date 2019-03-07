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

pub struct Label {
    w: f32,
    h: f32,
    text: String,
    text_renderer: Rc<text_renderer::TextRenderer>,
}

impl Label {
    pub fn new(w: f32, h: f32, text: &str,
        text_renderer: Rc<text_renderer::TextRenderer>) -> Rc<RefCell<Label>> {

        Rc::new(RefCell::new(Label{
            w, h, text: String::from(text), text_renderer
        }))
    }
}

impl UIElement for Label {
    fn get_size(&self) -> (f32, f32) {
        (self.w, self.h)
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        let aspect = (self.w / (gl_x1 - gl_x0)) / (self.h / (gl_y1 - gl_y0));
        self.text_renderer.render(&self.text, ((gl_x0 + gl_x1) * 0.5, (gl_y0 + gl_y1) * 0.5),
            gl_y1 - gl_y0, (0.0, 0.0, 0.0), aspect);

    }
    fn on_mouse_event(&mut self, _event: MouseEvent) -> Vec<UIEvent> {
        vec![]
    }
}

pub struct TextEdit {
    w: f32,
    h: f32,
    text: String,
    rect_renderer: Rc<rect_renderer::RectRenderer>,
    text_renderer: Rc<text_renderer::TextRenderer>,
}

impl TextEdit {
    pub fn new(w: f32, h: f32, rect_renderer: Rc<rect_renderer::RectRenderer>,
        text_renderer: Rc<text_renderer::TextRenderer>) -> Rc<RefCell<TextEdit>> {
        Rc::new(RefCell::new(TextEdit {
            w, h, text: String::from(""), rect_renderer, text_renderer
        }))
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl UIElement for TextEdit {
    fn get_size(&self) -> (f32, f32) {
        (self.w, self.h)
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        let aspect = (self.w / (gl_x1 - gl_x0)) / (self.h / (gl_y1 - gl_y0));
        self.text_renderer.render(&self.text, ((gl_x0 + gl_x1) * 0.5, (gl_y0 + gl_y1) * 0.5),
            gl_y1 - gl_y0, (0.0, 0.0, 0.0), aspect);

    }
    fn on_mouse_event(&mut self, _event: MouseEvent) -> Vec<UIEvent> {
        vec![]
    }
}

pub struct CheckBox {
    id: u32,
    width: f32,
    image_unchecked: Rc<texture::Texture>,
    image_checked: Rc<texture::Texture>,
    rect_renderer: Rc<rect_renderer::RectRenderer>,
    checked: bool,
    cursor_in: bool,
}

impl CheckBox {
    pub fn new(id: u32, width: f32,
        image_unchecked: Rc<texture::Texture>,
        image_checked: Rc<texture::Texture>,
        rect_renderer: Rc<rect_renderer::RectRenderer>) -> Rc<RefCell<CheckBox>> {
        Rc::new(RefCell::new(CheckBox{
            id,
            width,
            image_unchecked,
            image_checked,
            rect_renderer,
            checked: false,
            cursor_in: false,
        }))
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn get_checked(&self) -> bool {
        self.checked
    }
}

impl UIElement for CheckBox {
    fn get_size(&self) -> (f32, f32) {
        (self.width, self.width)
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        self.rect_renderer.render(((gl_x0, gl_y0), (gl_x1, gl_y1)),
            if self.cursor_in
                {rect_renderer::Filling::Color(1.0, 1.0, 0.5, 0.3)}
            else
                {rect_renderer::Filling::Color(1.0, 1.0, 1.0, 0.3)}
            );

            self.rect_renderer.render(((gl_x0, gl_y0), (gl_x1, gl_y1)),
                rect_renderer::Filling::Texture(
                    if self.checked {self.image_checked.as_ref()} else {self.image_unchecked.as_ref()},
                    ((0.0, 1.0), (1.0, 0.0)), (1.0, 1.0, 1.0)));
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

pub enum ButtonContent {
    Text(String),
    Image(texture::Texture),
}

impl ButtonContent {
    pub fn from_text(text: &str) -> ButtonContent {
        ButtonContent::Text(String::from(text))
    }

    pub fn from_image(data: &[u8]) -> ButtonContent {
        ButtonContent::Image(texture::Texture::from_png(data))
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
                    rect_renderer::Filling::Texture(&image, ((0.0, 1.0), (1.0, 0.0)), (1.0, 1.0, 1.0)));
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
    horizontal_count: usize,
    selected: usize,
    cursor_in: Option<usize>,
    colors: Vec<(u8, u8, u8)>,
    square_texture: texture::Texture,
    rect_renderer: Rc<rect_renderer::RectRenderer>,
}

impl Palette {
    pub fn new(id: u32, width: f32, horizontal_count: usize,
        rect_renderer: Rc<rect_renderer::RectRenderer>) -> Rc<RefCell<Palette>> {

        let texture = texture::Texture::from_png(include_bytes!("icon/white-square.png"));

        Rc::new(RefCell::new(Palette {
            id,
            width,
            horizontal_count,
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
        (self.width * self.horizontal_count as f32,
        self.width * (self.colors.len() / self.horizontal_count) as f32)
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        let vertical_count = self.colors.len() / self.horizontal_count;
        let width = (gl_x1 - gl_x0) / self.horizontal_count as f32;
        let height = (gl_y1 - gl_y0) / vertical_count as f32;
        for i in 0 .. self.colors.len() {
            let cx = i % self.horizontal_count;
            let cy = i / self.horizontal_count;
            let y0 = gl_y0 + height * (vertical_count - cy - 1) as f32;
            let y1 = y0 + height;
            let x0 = gl_x0 + width * cx as f32;
            let x1 = x0 + width;

            self.rect_renderer.render(((x0, y0), (x1, y1)),
            if self.selected == i
                {rect_renderer::Filling::Color(1.0, 0.5, 0.5, 0.3)}
            else if self.cursor_in == Some(i)
                {rect_renderer::Filling::Color(1.0, 1.0, 0.5, 0.3)}
            else
                {rect_renderer::Filling::Color(1.0, 1.0, 1.0, 0.3)}
            );

            self.rect_renderer.render(((x0, y0), (x1, y1)),
                rect_renderer::Filling::Texture(&self.square_texture, ((0.0, 1.0), (1.0, 0.0)),
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
            MouseEvent::Moved(x, y) => {
                let vertical_count = self.colors.len() / self.horizontal_count;
                let mut cy = y / self.width;
                let mut cx = x / self.width;
                if cx < 0.0 { cx = 0.0; }
                if cy < 0.0 { cy = 0.0; }
                let cxmax = (self.horizontal_count - 1) as f32;
                if cx > cxmax { cx = cxmax; }
                let cymax = (vertical_count - 1) as f32;
                if cy > cymax { cy = cymax; }
                self.cursor_in = Some(cx as usize + cy as usize * self.horizontal_count);
            }
            _ => ()
        }
        vec![]
    }
}

pub struct ScrollBar {
    id: u32,
    width: f32,
    height: f32,
    rect_renderer: Rc<rect_renderer::RectRenderer>,

    value: f32,
    cursor_in: bool,
    selected: bool,
    temp_value: f32,
}

impl ScrollBar {
    pub fn new(id: u32, width: f32, height: f32, rect_renderer: Rc<rect_renderer::RectRenderer>)
        -> Rc<RefCell<ScrollBar>> {
        Rc::new(RefCell::new(ScrollBar {
            id,
            width,
            height,
            rect_renderer,
            value: 0.0,
            cursor_in: false,
            selected: false,
            temp_value: 0.0,
        }))
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}


impl UIElement for ScrollBar {
    fn get_size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        let yc = (gl_y0 + gl_y1) * 0.5;
        let yh = (gl_y1 - gl_y0) * 0.4;
        self.rect_renderer.render(((gl_x0, yc - yh), (gl_x1, yc + yh)),
            if self.selected
                {rect_renderer::Filling::Color(1.0, 0.5, 0.5, 0.3)}
            else if self.cursor_in
                {rect_renderer::Filling::Color(1.0, 1.0, 0.5, 0.3)}
            else
                {rect_renderer::Filling::Color(1.0, 1.0, 1.0, 0.3)}
        );

        let tick_width = self.height * 0.2;
        let tick_width_gl = tick_width / self.width * (gl_x1 - gl_x0);
        let tick_pos = gl_x0 + (gl_x1 - gl_x0) * self.value;
        self.rect_renderer.render(((tick_pos - tick_width_gl, gl_y0), (tick_pos + tick_width_gl, gl_y1)),
                {rect_renderer::Filling::Color(0.2, 0.2, 0.5, 0.8)});

    }

    fn on_mouse_event(&mut self, event: MouseEvent) -> Vec<UIEvent> {
        match event {
            MouseEvent::Entered => {
                self.cursor_in = true;
            },
            MouseEvent::Left => {
                self.selected = false;
                self.cursor_in = false;
            }
            MouseEvent::Pressed => {
                self.selected = true;
                self.value = self.temp_value;
                return vec![UIEvent{id: self.id}]
            }
            MouseEvent::Released => {
                self.selected = false;
            }
            MouseEvent::Moved(x, _) => {
                self.temp_value = x / self.width;
                if self.selected {
                    self.value = self.temp_value;
                    return vec![UIEvent{id: self.id}]
                }
            }
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
    rect_renderer: Rc<rect_renderer::RectRenderer>,
    cursor_in: Option<usize>,
    visible: bool,
    color: Option<(f32, f32, f32, f32)>,
}

impl GridLayout {
    pub fn new(x_count: usize, y_count: usize, children: Vec<Rc<RefCell<dyn UIElement>>>,
        xl_margin: f32, xr_margin: f32, yt_margin: f32, yb_margin: f32, x_gap: f32, y_gap: f32,
        rect_renderer: Rc<rect_renderer::RectRenderer>)
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
            rect_renderer,
            cursor_in: None,
            visible: true,
            color: None,
        }))
    }

    pub fn set_color(&mut self, color: (f32, f32, f32, f32)) {
        self.color = Some(color);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn get_visible(&self) -> bool {
        self.visible
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
        if !self.visible {
            return (0.0, 0.0);
        }
        let w_base = self.xl_margin + self.xr_margin + self.x_gap * (self.x_count - 1) as f32;
        let h_base = self.yt_margin + self.yb_margin + self.y_gap * (self.y_count - 1) as f32;
        let (ws, hs) = self.get_grid_size();

        (w_base + ws.iter().fold(0.0, |acc, a|acc + a), h_base + hs.iter().fold(0.0, |acc, a|acc + a))
    }
    fn render(&self, (gl_x0, gl_y0): (f32, f32), (gl_x1, gl_y1): (f32, f32)) {
        if !self.visible {
            return;
        }

        if let Some((r, g, b, a)) = self.color {
            self.rect_renderer.render(((gl_x0, gl_y0), (gl_x1, gl_y1)),
                rect_renderer::Filling::Color(r, g, b, a));
        }

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
                    ui_event.append(&mut self.children[current].borrow_mut().on_mouse_event(MouseEvent::Moved(dx, dy)));
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
