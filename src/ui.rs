use glium::{Surface, DrawParameters, Blend, Display, Frame};
use glium::glutin::event::{MouseButton, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};
use glium::glutin::window::CursorIcon;

use clipboard::{ClipboardProvider, ClipboardContext};

use crate::render::{Canvas, Vertex};
use crate::font::{FontParameters, TextAlignVertical, TextAlignHorizontal};
use crate::window::{WindowListener, Window};
use image::DynamicImage;
use std::thread::JoinHandle;
use std::str::FromStr;
use cgmath::{Vector2, InnerSpace, MetricSpace};
use glium::index::PrimitiveType;
use std::any::Any;
use std::time::Instant;

pub struct Widgets<S> where S: Surface {
    widgets: Vec<Box<dyn Widget<S>>>,
    focus: usize
}

impl<S> Widgets<S> where S: Surface {
    pub fn new() -> Widgets<S> {
        Widgets {
            widgets: Vec::new(),
            focus: 0
        }
    }

    pub fn get(&self, id: usize) -> Option<&Box<dyn Widget<S>>> {
        self.widgets.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Box<dyn Widget<S>>> {
        self.widgets.get_mut(id)
    }

    pub fn find<I, W>(&self, id: I) -> Option<&W> where I: AsRef<str>, W: Widget<S> + 'static {
        let id = id.as_ref();
        let w = self.widgets.iter().find(|w| w.get_id() == id);
        if let Some(w) = w {
            match (&**w).as_any().downcast_ref::<W>() {
                Some(w) => return Some(w),
                _ => {}
            }
        }
        None
    }

    pub fn find_mut<I, W>(&mut self, id: I) -> Option<&mut W> where I: AsRef<str>, W: Widget<S> + 'static {
        let id = id.as_ref();
        let w = self.widgets.iter_mut().find(|w| w.get_id() == id);
        if let Some(w) = w {
            match (&mut **w).as_mut_any().downcast_mut::<W>() {
                Some(w) => return Some(w),
                _ => {}
            }
        }
        None
    }

    pub fn add<W>(&mut self, widget: W) where W: 'static + Widget<S> {
        self.widgets.push(Box::new(widget));
    }

    pub fn tab_focus(&mut self, next: bool) -> Vec<WidgetEvent> {
        let mut events = Vec::new();

        let prev = self.focus % self.widgets.len();
        let new = (self.focus as isize + if next { 1 } else { -1 }) % self.widgets.len() as isize;
        let new = if new < 0 { self.widgets.len() as isize + new } else { new } as usize;
        if prev != new {
            events.extend(self.change_focus(Some(new)));
        }

        events
    }

    fn change_focus(&mut self, id: Option<usize>) -> Vec<WidgetEvent> {
        let mut events = Vec::new();

        if let Some(id) = id {
            for (i, e) in self.widgets.iter_mut().enumerate() {
                let focus = i == id;
                if focus != e.is_focused() {
                    e.set_focused(focus);
                    events.push(WidgetEvent::FocusChanged { id: e.get_id().clone(), focus });
                }
            }
            self.focus = id;
        }
        events
    }

    pub fn update(&mut self, mouse_pos: (f32, f32), partial_ticks: f32) {
        for e in self.widgets.iter_mut() {
            e.update(mouse_pos, partial_ticks);
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<S>, partial_ticks: f32) {
        for e in self.widgets.iter() {
            e.draw(canvas, partial_ticks);
        }
    }

    fn propagate_event<P>(&mut self, propagator: P) -> Vec<WidgetEvent> where P: Fn(&mut dyn Widget<S>) -> Vec<WidgetEvent> {
        let mut events = Vec::new();
        let mut focus = None;
        for (i, e) in self.widgets.iter_mut().enumerate() {
            for event in propagator(&mut **e) {
                if let WidgetEvent::FocusChanged { id, focus: f } = event {
                    if f {
                        focus = Some(i);
                    }
                } else {
                    events.push(event);
                }
            }
        }
        events.extend(self.change_focus(focus));
        events
    }

    pub fn on_keyboard_char(&mut self, display: &Display, ch: char) -> Vec<WidgetEvent> {
        self.propagate_event(move |e| e.on_keyboard_char(ch))
    }

    pub fn on_keyboard_key(&mut self, display: &Display, input: KeyboardInput) -> Vec<WidgetEvent> {
        self.propagate_event(move |e| e.on_keyboard_key(input))
    }

    pub fn on_mouse_button(&mut self, display: &Display, button: MouseButton,
                           state: ElementState, pos: (f32, f32)) -> Vec<WidgetEvent> {
        self.propagate_event(move |e| e.on_mouse_button(button, state, pos))
    }

    pub fn on_mouse_wheel(&mut self, display: &Display, delta: MouseScrollDelta) -> Vec<WidgetEvent> {
        self.propagate_event(move |e| e.on_mouse_wheel(delta))
    }

    pub fn on_mouse_move(&mut self, display: &Display, pos: (f32, f32)) -> Vec<WidgetEvent> {
        self.propagate_event(move |e| e.on_mouse_move(pos))
    }

    pub fn get_cursor(&self, mouse_pos: (f32, f32)) -> CursorIcon {
        for e in self.widgets.iter().rev() {
            if Widget::<S>::is_mouse_over(&**e, mouse_pos) {
                if let Some(cursor) = (*e).get_cursor(mouse_pos) {
                    return cursor;
                }
            }
        }
        CursorIcon::Default
    }
}

pub trait Widget<S> where S: Surface {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn get_id(&self) -> &String;
    fn get_bounds(&self) -> (f32, f32, f32, f32);
    fn get_cursor(&self, mouse: (f32, f32)) -> Option<CursorIcon> {
        None
    }
    fn is_mouse_over(&self, mouse: (f32, f32)) -> bool {
        let (mouse_x, mouse_y) = mouse;
        let (x, y, w, h) = self.get_bounds();
        mouse_x >= x && mouse_x <= (x + w) && mouse_y >= y && mouse_y <= (y + h)
    }
    fn is_focused(&self) -> bool;
    fn set_focused(&mut self, focused: bool);
    fn on_mouse_button(&mut self, button: MouseButton, state: ElementState, pos: (f32, f32)) -> Vec<WidgetEvent> { vec![] }
    fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) -> Vec<WidgetEvent> { vec![] }
    fn on_mouse_move(&mut self, pos: (f32, f32)) -> Vec<WidgetEvent> { vec![] }
    fn on_keyboard_key(&mut self, input: KeyboardInput) -> Vec<WidgetEvent> { vec![] }
    fn on_keyboard_char(&mut self, ch: char) -> Vec<WidgetEvent> { vec![] }
    fn update(&mut self, mouse_pos: (f32, f32), partial_ticks: f32) {}
    fn draw(&self, canvas: &mut Canvas<S>, partial_ticks: f32) where S: Surface;
}

#[derive(Clone)]
pub enum Background {
    Texture(String),
    Color([f32; 4])
}

impl Background {
    pub fn draw<S>(&self, canvas: &mut Canvas<S>, bounds: [f32; 4], partial_ticks: f32) where S: Surface {
        let viewport: [[f32; 4]; 4] = canvas.viewport().into();
        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            .. Default::default()
        };
        match self {
            Background::Texture(texture) => {
                let texture = canvas.textures().borrow().get(texture);
                let program = canvas.shaders().borrow().textured();
                let uniforms = uniform! {
                    mat: viewport,
                    tex: texture.sampled()
                };
                canvas.textured_rect(bounds, [1.0; 4], &program, &uniforms, &params);
            },
            Background::Color(color) => {
                let program = canvas.shaders().borrow().default();
                let uniforms = uniform! {
                    mat: viewport
                };
                canvas.rect(bounds, *color, &program, &uniforms, &params);
            },
        }
    }
}

pub struct Button {
    id: String,
    label: String,
    bounds: (f32, f32, f32, f32),
    pressed: bool,
    hover: bool,
    focused: bool,
    background_normal: Background,
    background_hover: Background
}

impl<S> Widget<S> for Button where S: Surface {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_bounds(&self) -> (f32, f32, f32, f32) {
        self.bounds
    }

    fn get_cursor(&self, mouse: (f32, f32)) -> Option<CursorIcon> {
        if Widget::<S>::is_mouse_over(self, mouse) {
            Some(CursorIcon::Hand)
        } else {
            None
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn on_mouse_button(&mut self, button: MouseButton, state: ElementState, pos: (f32, f32)) -> Vec<WidgetEvent> {
        let mut clicked = false;
        if self.hover {
            match button {
                MouseButton::Left => {
                    if state == ElementState::Pressed {
                        self.pressed = true;
                        return vec![WidgetEvent::FocusChanged { id: Widget::<S>::get_id(self).clone(), focus: true }];
                    } else {
                        self.pressed = false;
                        clicked = true;
                    }
                },
                _ => {}
            }
        }
        if clicked {
            return vec![WidgetEvent::ButtonClicked { id: Widget::<S>::get_id(self).clone() }];
        }
        vec![]
    }

    fn on_mouse_move(&mut self, pos: (f32, f32)) -> Vec<WidgetEvent> {
        self.hover = Widget::<S>::is_mouse_over(self, pos);
        vec![]
    }

    fn on_keyboard_key(&mut self, input: KeyboardInput) -> Vec<WidgetEvent> {
        let KeyboardInput { virtual_keycode, state, .. } = input;
        if self.focused && Some(VirtualKeyCode::Return) == virtual_keycode {
            if state == ElementState::Pressed {
                self.pressed = true;
            } else {
                self.pressed = false;
                return vec![WidgetEvent::ButtonClicked{ id: Widget::<S>::get_id(self).clone() }]
            }
        }
        vec![]
    }

    fn draw(&self, canvas: &mut Canvas<S>, partial_ticks: f32) {
        let (x, y, w, h) = Widget::<S>::get_bounds(self);
        let bounds = [x, y, w, h];
        let background = if self.hover { &self.background_hover } else { &self.background_normal };
        background.draw(canvas, bounds, partial_ticks);
        canvas.text(&self.label, x + w / 2.0, y + h / 4.0, &FontParameters {
            color: [1.0; 4],
            align_horizontal: TextAlignHorizontal::Center,
            align_vertical: TextAlignVertical::Center,
            .. Default::default()
        });
    }
}

impl Button {
    pub fn new<I, T>(id: I, label: T, x: f32, y: f32, w: f32, h: f32, background_normal: Background,
                     background_hover: Option<Background>) -> Button
        where I: Into<String>, T: Into<String> {

        Button {
            id: id.into(),
            label: label.into(),
            bounds: (x, y, w, h),
            pressed: false,
            hover: false,
            focused: false,
            background_hover: background_hover.unwrap_or_else(|| background_normal.clone()),
            background_normal
        }
    }
}

pub type TextMask = dyn Fn(&String, bool) -> String + 'static + Send + Sync;

pub struct TextField {
    id: String,
    placeholder: String,
    value: String,
    filter: Option<TextFilter>,
    mask: Option<Box<TextMask>>,
    focused: bool,
    bounds: (f32, f32, f32, f32),
    background: Background,
    last_input_changed: Instant
}

impl<S> Widget<S> for TextField where S: Surface {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_bounds(&self) -> (f32, f32, f32, f32) {
        self.bounds
    }

    fn get_cursor(&self, mouse: (f32, f32)) -> Option<CursorIcon> {
        if Widget::<S>::is_mouse_over(self, mouse) {
            Some(CursorIcon::Text)
        } else {
            None
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn on_mouse_button(&mut self, button: MouseButton, state: ElementState, pos: (f32, f32)) -> Vec<WidgetEvent> {
        if Widget::<S>::is_mouse_over(self, pos) {
            if button == MouseButton::Left && state == ElementState::Pressed {
                self.focused = true;
                return vec![WidgetEvent::FocusChanged { id: Widget::<S>::get_id(self).clone(), focus: true }];
            }
        } else {
            self.focused = false;
        }
        vec![]
    }

    fn on_keyboard_key(&mut self, input: KeyboardInput) -> Vec<WidgetEvent> {
        let KeyboardInput { virtual_keycode, state, modifiers, .. } = input;
        if self.focused && state == ElementState::Pressed {
            match virtual_keycode {
                Some(VirtualKeyCode::Back) => {
                    if !self.value.is_empty() {
                        self.value.pop();
                        self.last_input_changed = Instant::now();
                        return vec![WidgetEvent::TextValueChanged {
                            id: Widget::<S>::get_id(self).clone(), value: self.value.clone()
                        }];
                    }
                },
                Some(VirtualKeyCode::Escape) => { self.focused = false; },
                Some(VirtualKeyCode::Delete) => {
                    self.value.clear();
                    self.last_input_changed = Instant::now();
                },
                Some(VirtualKeyCode::V) => {
                    if modifiers.ctrl() {
                        let mut clipboard: ClipboardContext = ClipboardProvider::new().expect("Failed to access clipboard");
                        let contents = clipboard.get_contents().expect("Failed to get clipboard contents");
                        self.value.push_str(&contents);
                        self.last_input_changed = Instant::now();
                        return vec![WidgetEvent::TextValueChanged {
                            id: Widget::<S>::get_id(self).clone(), value: self.value.clone()
                        }];
                    }
                }
                _ => {}
            }
        }
        vec![]
    }

    fn on_keyboard_char(&mut self, ch: char) -> Vec<WidgetEvent> {
        if self.focused && (ch == ' ' || !ch.is_control()) {
            if let Some(filter) = &self.filter {
                if !filter.matches(ch, &self.value) {
                    return vec![];
                }
            }
            self.value.push(ch);
            self.last_input_changed = Instant::now();
            return vec![WidgetEvent::TextValueChanged {
                id: Widget::<S>::get_id(self).clone(), value: self.value.clone()
            }];
        }
        vec![]
    }

    fn draw(&self, canvas: &mut Canvas<S>, partial_ticks: f32) {
        let (x, y, w, h) = Widget::<S>::get_bounds(self);
        let bounds = [x, y, w, h];
        let default_program = canvas.shaders().borrow().default();
        let viewport: [[f32; 4]; 4] = canvas.viewport().into();
        let uniforms = uniform! {
            mat: viewport
        };
        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            line_width: Some(1.0), //FIXME 1.2
            .. Default::default()
        };
        self.background.draw(canvas, bounds, partial_ticks);
        let mut text = self.get_display_text();
        let (mut text_w, text_h) = canvas.get_text_size(&text, &Default::default());
        while text_w > w - 10.0 {
            if self.focused {
                text.remove(0);
            } else {
                text.pop();
            }
            let (w, _) = canvas.get_text_size(&text, &Default::default());
            text_w = w;
        }
        let font_params = FontParameters {
            color: if self.value.is_empty() { [0.2, 0.2, 0.2, 1.0] } else { [1.0; 4] },
            align_horizontal: TextAlignHorizontal::Left,
            align_vertical: TextAlignVertical::Center,
            .. Default::default()
        };
        canvas.text(text, x + 5.0, y + h / 4.0, &font_params);
        if self.focused && Instant::now().duration_since(self.last_input_changed).subsec_millis() < 500 {
            let offset = if self.value.is_empty() { 0.0 } else { text_w } + 4.0;
            canvas.rect([x + offset, y + 2.0, 2.0, h - 4.0], [1.0; 4], &default_program, &uniforms, &params);
        }
    }
}

impl TextField {
    pub fn new<I, P, V>(id: I, placeholder: P, value: V, x: f32, y: f32, w: f32, h: f32, background: Background,
                           filter: Option<TextFilter>, mask: Option<Box<TextMask>>) -> TextField
        where I: Into<String>, P: Into<String>, V: Into<String> {

        TextField {
            id: id.into(),
            placeholder: placeholder.into(),
            value: value.into(),
            filter,
            mask,
            focused: false,
            bounds: (x, y, w, h),
            background,
            last_input_changed: Instant::now()
        }
    }

    fn get_display_text(&self) -> String {
        if self.value.is_empty() {
            self.placeholder.clone()
        } else if let Some(ref mask) = self.mask {
            (*mask)(&self.value, self.focused)
        } else {
            self.value.clone()
        }
    }
}

pub struct ScrollBar {
    id: String,
    steps: u32,
    value: f32,
    max: f32,
    focused: bool,
    bounds: (f32, f32, f32, f32),
    color: [f32; 4]
}

impl<S> Widget<S> for ScrollBar where S: Surface {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_bounds(&self) -> (f32, f32, f32, f32) {
        self.bounds
    }

    fn get_cursor(&self, mouse: (f32, f32)) -> Option<CursorIcon> {
        if Widget::<S>::is_mouse_over(self, mouse) {
            Some(CursorIcon::Hand)
        } else {
            None
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn on_mouse_button(&mut self, button: MouseButton, state: ElementState, pos: (f32, f32)) -> Vec<WidgetEvent> {
        if Widget::<S>::is_mouse_over(self, pos) {
            if button == MouseButton::Left {
                if state == ElementState::Pressed {
                    self.focused = true;
                    let (mouse_x, mouse_y) = pos;
                    let (x, y, w, h) = Widget::<S>::get_bounds(self);
                    let value = ((mouse_x - x) / w * self.max).max(0.0).min(self.max);
                    self.value = value;
                    let id = Widget::<S>::get_id(self).clone();
                    return vec![
                        WidgetEvent::FocusChanged { id: id.clone(), focus: true },
                        WidgetEvent::ScrollValueChanged { id, value: self.value, max: self.max, steps: self.steps }
                    ];
                } else if state == ElementState::Released {
                    self.focused = false;
                    let id = Widget::<S>::get_id(self).clone();
                    return vec![
                        WidgetEvent::FocusChanged { id: id.clone(), focus: false },
                    ];
                }
            }
        } else {
            self.focused = false;
        }
        vec![]
    }

    fn on_mouse_move(&mut self, pos: (f32, f32)) -> Vec<WidgetEvent> {
        if self.focused {
            let (mouse_x, mouse_y) = pos;
            let (x, y, w, h) = Widget::<S>::get_bounds(self);
            let value = ((mouse_x - x) / w * self.max).max(0.0).min(self.max);
            if value != self.value {
                let id = Widget::<S>::get_id(self).clone();
                self.value = value;
                return vec![
                    WidgetEvent::ScrollValueChanged { id, value: self.value, max: self.max, steps: self.steps }
                ];
            }
        }
        vec![]
    }

    fn draw(&self, canvas: &mut Canvas<S>, partial_ticks: f32) {
        let (x, y, w, h) = Widget::<S>::get_bounds(self);
        let bounds = [x, y, w, h];
        let default_program = canvas.shaders().borrow().default();
        let viewport: [[f32; 4]; 4] = canvas.viewport().into();
        let uniforms = uniform! {
            mat: viewport
        };
        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        let sp = self.value / self.max;
        let sw = w / (self.steps as f32 + 1.0);
        let sx = (w * sp - sw / 2.0).max(0.0).min(w - sw);

        if sx > 0.0 {
            canvas.rect([x, y + h / 4.0, sx, h / 2.0], self.color, &default_program, &uniforms, &params);
        }
        if sx < w - sw {
            canvas.rect([x + sx + sw, y + h / 4.0, w - sx - sw, h / 2.0], self.color, &default_program, &uniforms, &params);
        }
        canvas.rect([x + sx, y, sw, h], self.color, &default_program, &uniforms, &params);
    }
}

impl ScrollBar {
    pub fn new<I, C>(id: I, value: f32, max: f32, steps: u32, x: f32, y: f32, w: f32, h: f32, color: C) -> ScrollBar
        where I: Into<String>, C: Into<[f32;4]> {

        ScrollBar {
            id: id.into(),
            value, max, steps,
            focused: false,
            bounds: (x, y, w, h),
            color: color.into()
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.min(self.max).max(0.0);
    }

    pub fn set_ratio_value(&mut self, ratio: f32) {
        self.value = ratio.max(0.0).min(1.0) * self.max;
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }
}

struct Notification {
    size: (u32, u32),
    widgets: Widgets<Frame>,
    message: String,
    visible: bool
}

pub fn notification<S, T, M>(size: S, title: T, message: M, icon: Option<DynamicImage>, tps: u32) -> JoinHandle<()>
    where S: Into<(u32, u32)>, T: Into<String>, M: Into<String> {

    let (w, h) = size.into();
    let title = title.into();
    let message = message.into();

    std::thread::Builder::new().name("gui-notification".to_string()).spawn(move || {
        let mut dialog = Notification {
            size: (w, h),
            widgets: Widgets::new(),
            message,
            visible: true
        };

        Window::show((w, h), title, icon, true, false, true, true, &mut dialog, tps)
    }).expect("thread spawning failed")
}

impl WindowListener for Notification {
    fn is_closed(&self, display: &Display) -> bool {
        self.visible
    }

    fn on_created(&mut self, display: &Display) {
        let (w, h) = self.size;

        self.widgets.add(Button::new("dispose_btn",
                                     "Ок", w as f32 / 2.0 - 40.0, h as f32 - 40.0, 80.0, 30.0,
            Background::Color([0.3, 0.3, 0.3, 0.8]), None
        ));
    }

    fn on_frame_draw(&self, canvas: &mut Canvas<Frame>, mouse_pos: (f32, f32), partial_ticks: f32) {
        canvas.clear((1.0, 1.0, 1.0, 1.0), 1.0);
        canvas.text(&self.message, self.size.0 as f32 / 2.0, 10.0, &Default::default());
        self.widgets.draw(canvas, partial_ticks);
    }

    fn on_close_requested(&mut self, display: &Display, dimensions: (f32, f32)) {
        self.visible = false;
    }

    fn on_keyboard_char(&mut self, display: &Display, dimensions: (f32, f32), ch: char) {
        for e in self.widgets.on_keyboard_char(display, ch) {
            self.on_widget_event(display, e);
        }
    }

    fn on_keyboard_key(&mut self, display: &Display, dimensions: (f32, f32), input: KeyboardInput) {
        for e in self.widgets.on_keyboard_key(display, input) {
            self.on_widget_event(display, e);
        }
    }

    fn on_mouse_button(&mut self, display: &Display, dimensions: (f32, f32), button: MouseButton, state: ElementState, pos: (f32, f32)) {
        for e in self.widgets.on_mouse_button(display, button, state, pos) {
            self.on_widget_event(display, e);
        }
    }

    fn on_mouse_wheel(&mut self, display: &Display, dimensions: (f32, f32), delta: MouseScrollDelta) {
        for e in self.widgets.on_mouse_wheel(display, delta) {
            self.on_widget_event(display, e);
        }
    }

    fn on_mouse_move(&mut self, display: &Display, dimensions: (f32, f32), pos: (f32, f32)) {
        for e in self.widgets.on_mouse_move(display, pos) {
            self.on_widget_event(display, e);
        }
    }
}

impl Notification {
    fn on_widget_event(&mut self, display: &Display, event: WidgetEvent) {
        match event {
            WidgetEvent::ButtonClicked { id } => {
                if id == "dispose_btn" {
                    self.visible = false;
                }
            },
            _ => {}
        }
    }
}

pub enum WidgetEvent {
    ButtonClicked { id: String },
    TextValueChanged { id: String, value: String },
    ScrollValueChanged { id: String, value: f32, max: f32, steps: u32 },
    FocusChanged { id: String, focus: bool }
}

pub fn is_valid_number<N: FromStr>(c: char, v: &String) -> bool {
    if !c.is_numeric() {
        false
    } else {
        let mut v = v.clone();
        v.push(c);
        v.parse::<N>().is_ok()
    }
}

pub struct TextFilter {
    pub filter: Box<dyn Fn(char, &String) -> bool + 'static + Send + Sync>
}

impl TextFilter {
    pub fn numeric<N: FromStr>() -> TextFilter where N: 'static {
        TextFilter { filter: Box::new(is_valid_number::<N>) }
    }

    pub fn matches(&self, c: char, v: &String) -> bool {
        (*self.filter)(c, v)
    }
}