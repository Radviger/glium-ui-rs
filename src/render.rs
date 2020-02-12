use std::ops::Mul;
use std::rc::Rc;
use std::cell::RefCell;

use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer, Display, DrawParameters, Surface, Program, Rect};
use glium::uniforms::Uniforms;
use cgmath::{Matrix4, Point3, Transform};

use crate::font::{FontManager, FontParameters};
use crate::shader::ShaderManager;
use crate::texture::TextureManager;
use winit::dpi::LogicalSize;

pub struct DrawBuffer {
    capacity: usize,
    index: u32,
    vertices: Vec<WrappedVertex>,
    indices: Vec<u32>,
    normal: bool,
    texture: bool,
    primitive_type: Option<PrimitiveType>,
    drawing: bool
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: Option<[f32; 3]>,
    pub color: Option<[f32; 4]>,
    pub texture_uv: Option<[f32; 2]>
}

impl Vertex {
    pub fn pos<P: Into<[f32; 3]>>(pos: P) -> Vertex {
        Vertex {
            pos: pos.into(),
            normal: None,
            color: None,
            texture_uv: None,
        }
    }

    pub fn normal<N: Into<[f32; 3]>>(mut self, normal: N) -> Self {
        self.normal = Some(normal.into());
        self
    }

    pub fn color<C: Into<[f32; 4]>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn uv(mut self, texture_uv: [f32; 2]) -> Self {
        self.texture_uv = Some(texture_uv);
        self
    }
}

impl Mul<Matrix4<f32>> for Vertex {
    type Output = Self;

    fn mul(self, mat: Matrix4<f32>) -> Self::Output {
        let pos = mat.transform_point(Point3::from(self.pos)).into();
        Vertex {
            pos,
            .. self
        }
    }
}

impl DrawBuffer {
    pub fn new() -> DrawBuffer {
        Self::with_capacity(0)
    }

    pub fn draw_once<S, U>(primitive_type: &PrimitiveType, normal: bool, texture: bool,
                           display: &Display, target: &mut S, program: &glium::Program, uniform: &U,
                           params: &DrawParameters, vertices: Vec<Vertex>)
        where S: Surface, U: Uniforms {

        let mut buffer = Self::with_capacity(vertices.len());
        buffer.start_drawing(primitive_type, normal, texture);

        for vertex in vertices {
            buffer.add_vertex(vertex);
        }

        buffer.draw(display, target, program, uniform, params);
    }

    pub fn with_capacity(initial_capacity: usize) -> DrawBuffer {
        DrawBuffer {
            capacity: initial_capacity,
            index: 0,
            vertices: Vec::with_capacity(initial_capacity),
            indices: Vec::with_capacity(initial_capacity),
            normal: false,
            texture: false,
            primitive_type: None,
            drawing: false
        }
    }

    pub fn start_drawing(&mut self, primitive_type: &PrimitiveType, normal: bool, texture: bool) {
        if !self.drawing {
            self.drawing = true;
            self.primitive_type = Some(*primitive_type);
            self.normal = normal;
            self.texture = texture;
        } else {
            panic!("Already drawing!");
        }
    }

    pub fn draw<U, S>(&self, display: &Display, target: &mut S, program: &glium::Program, uniform: &U, params: &DrawParameters)
        where U: glium::uniforms::Uniforms,
              S: Surface {

        if self.drawing {
            let ib = IndexBuffer::new(display, self.primitive_type.expect("Getting primitive type failed"), &self.indices).expect("IndexBuffer creation failed");
            if self.texture {
                let mut vertices: Vec<TexturedVertex> = Vec::with_capacity(self.vertices.capacity());
                for v in &self.vertices {
                    match v {
                        &WrappedVertex::Textured(vtx) => vertices.push(vtx),
                        _ => panic!("Illegal buffer state")
                    }
                }
                let vb = VertexBuffer::new(display, &vertices).expect("VertexBuffer creation failed");
                target.draw(&vb, &ib, program, uniform, params).expect("Target drawing failed");
            } else {
                let mut vertices: Vec<SimpleVertex> = Vec::with_capacity(self.vertices.capacity());
                for v in &self.vertices {
                    match v {
                        &WrappedVertex::Simple(vtx) => vertices.push(vtx),
                        _ => panic!("Illegal buffer state")
                    }
                }
                let vb = VertexBuffer::new(display, &vertices).expect("VertexBuffer creation failed");
                target.draw(&vb, &ib, program, uniform, params).expect("Target drawing failed");
            };
        } else {
            panic!("Not drawing!")
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.vertices.clear();
        self.indices.clear();
        self.primitive_type = None;
        self.drawing = false;
    }

    pub fn add_multiple_vertices(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        if !self.drawing {
            panic!("Not drawing!");
        }
        let offset = vertices.len() as u32;
        for vertex in vertices {
            let pos = vertex.pos;
            let normal = vertex.normal;
            let color = vertex.color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
            let texture_uv = vertex.texture_uv;

            if !self.normal && normal.is_some() {
                panic!("Normal is not enabled for current drawing stage");
            }

            if !self.texture && texture_uv.is_some() {
                panic!("Texture is not enabled for current drawing stage");
            }

            if self.texture {
                self.vertices.push(WrappedVertex::Textured(TexturedVertex {
                    pos, color, normal: normal.unwrap_or([1.0, 1.0, 1.0]), texture_uv: texture_uv.expect("Texture uv getting failed")
                }));
            } else {
                self.vertices.push(WrappedVertex::Simple(SimpleVertex {
                    pos, normal: normal.unwrap_or([1.0, 1.0, 1.0]), color
                }))
            }
        }
        for index in indices {
            if index < offset {
                self.indices.push(self.index + index);
            } else {
                panic!("Illegal buffer index {} while amount of vertices is {}", index, offset);
            }
        }
        self.index += offset;
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.add_multiple_vertices(vec![vertex], vec![0]);
    }
}

enum WrappedVertex {
    Simple(SimpleVertex),
    Textured(TexturedVertex),
}

#[derive(Copy, Clone)]
struct SimpleVertex {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4]
}

implement_vertex!(SimpleVertex, pos, normal, color);

#[derive(Copy, Clone)]
struct TexturedVertex {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    texture_uv: [f32; 2]
}

implement_vertex!(TexturedVertex, pos, normal, color, texture_uv);

/*pub struct AnimationFrame {
    left: u32,
    top: u32,
    delay: f32
}

pub struct Animation {
    texture: Texture2d,
    frames: AnimationFrame,
    current_frame: usize
}

impl Animation {
    pub fn new<R>(r: R) -> ImageResult<Animation> where R: Read {
        let mut decoder = image::gif::Decoder::new(r);
        let raw_frames = decoder.into_frames()?;
        let mut frames = Vec::new();
        let mut pixels = Vec::new();
        for frame in raw_frames {
            frames.push(AnimationFrame {
                left: frame.left(),
                top: frame.top(),
                delay: frame.delay().numer() as f32 / frame.delay().denom() as f32
            });
        }
    }
}*/

pub struct Canvas<S> where S: Surface {
    display: Display,
    shaders: Rc<RefCell<ShaderManager>>,
    fonts: Rc<RefCell<FontManager>>,
    textures: Rc<RefCell<TextureManager>>,
    target: S
}

impl<S> Canvas<S> where S: Surface {
    pub fn new(display: Display, shaders: Rc<RefCell<ShaderManager>>, fonts: Rc<RefCell<FontManager>>,
               textures: Rc<RefCell<TextureManager>>, target: S) -> Canvas<S> {
        Canvas { display, shaders, fonts, textures, target }
    }

    pub fn display(&self) -> Display {
        self.display.clone()
    }

    pub fn shaders(&self) -> Rc<RefCell<ShaderManager>> {
        self.shaders.clone()
    }

    pub fn fonts(&self) -> Rc<RefCell<FontManager>> {
        self.fonts.clone()
    }

    pub fn textures(&self) -> Rc<RefCell<TextureManager>> {
        self.textures.clone()
    }

    pub fn dimensions(&self) -> (f32, f32) {
        let factor = self.scale_factor();
        let (w, h) = self.target.get_dimensions();
        (w as f32 / factor, h as f32 / factor)
    }

    pub fn scale_factor(&self) -> f32 {
        self.display.gl_window().window().scale_factor() as f32
    }

    pub fn viewport(&self) -> Matrix4<f32> {
        let (w, h) = self.dimensions();
        cgmath::ortho(0.0, w, h, 0.0, -0.1, 0.1)
    }

    pub fn scissor<B>(&self, bounds: B) -> Rect where B: Into<[f32; 4]> {
        let [x, y, w, h] = bounds.into();
        let (_, canvas_h) = self.dimensions();
        let factor = self.scale_factor();
        Rect {
            left: (x * factor).round() as u32, bottom: ((canvas_h - (y + h)) * factor).round() as u32,
            width: (w * factor).round() as u32, height: (h * factor).round() as u32
        }
    }

    pub fn clear(&mut self, color: (f32, f32, f32, f32), depth: f32) {
        self.target.clear_color_and_depth(color, depth);
    }

    pub fn rect<B, C, U>(&mut self, bounds: B, color: C, program: &Program, uniforms: &U,
                         params: &DrawParameters)
        where B: Into<[f32; 4]>, C: Into<[f32; 4]>, U: Uniforms {

        let bounds = bounds.into();
        let color = color.into();

        DrawBuffer::draw_once(
            &PrimitiveType::TriangleFan, false, false, &self.display.clone(),
            &mut self.target, program, uniforms, params,
            vec! [
                Vertex::pos([bounds[0], bounds[1], 0.0]).color(color),
                Vertex::pos([bounds[0] + bounds[2], bounds[1], 0.0]).color(color),
                Vertex::pos([bounds[0] + bounds[2], bounds[1] + bounds[3], 0.0]).color(color),
                Vertex::pos([bounds[0], bounds[1] + bounds[3], 0.0]).color(color),
            ]
        )
    }

    pub fn frame<B, C, U>(&mut self, bounds: B, color: C, program: &Program, uniforms: &U,
                          params: &DrawParameters)
        where B: Into<[f32; 4]>, C: Into<[f32; 4]>, U: Uniforms {

        let bounds = bounds.into();
        let color = color.into();

        DrawBuffer::draw_once(
            &PrimitiveType::LineLoop, false, false, &self.display.clone(),
            &mut self.target, program, uniforms, params,
            vec! [
                Vertex::pos([bounds[0], bounds[1], 0.0]).color(color),
                Vertex::pos([bounds[0] + bounds[2], bounds[1], 0.0]).color(color),
                Vertex::pos([bounds[0] + bounds[2], bounds[1] + bounds[3], 0.0]).color(color),
                Vertex::pos([bounds[0], bounds[1] + bounds[3], 0.0]).color(color),
            ]
        )
    }

    pub fn textured_rect<B, C, U>(&mut self, bounds: B, color: C, program: &Program, uniforms: &U,
                                  params: &DrawParameters)
        where B: Into<[f32; 4]>, C: Into<[f32; 4]>, U: Uniforms {

        let bounds = bounds.into();
        let color = color.into();

        DrawBuffer::draw_once(
            &PrimitiveType::TriangleFan, false, true, &self.display.clone(),
            &mut self.target, program, uniforms, params,
            vec! [
                Vertex::pos([bounds[0], bounds[1], 0.0]).color(color).uv([0.0, 0.0]),
                Vertex::pos([bounds[0] + bounds[2], bounds[1], 0.0]).color(color).uv([1.0, 0.0]),
                Vertex::pos([bounds[0] + bounds[2], bounds[1] + bounds[3], 0.0]).color(color).uv([1.0, 1.0]),
                Vertex::pos([bounds[0], bounds[1] + bounds[3], 0.0]).color(color).uv([0.0, 1.0]),
            ]
        )
    }

    pub fn fill_textured_rect<T, B, C>(&mut self, texture: T, bounds: B, color: C, program: &Program,
                                       params: &DrawParameters)
        where B: Into<[f32; 4]>, C: Into<[f32; 4]>, T: AsRef<str> {

        let texture = self.textures().borrow().get(texture);
        let mat = self.viewport();

        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(mat),
            tex: texture.sampled()
        };

        self.textured_rect(bounds, color, program, &uniforms, &params);
    }

    pub fn generic_shape<U>(&mut self, ty: &PrimitiveType, vertices: Vec<Vertex>, texture: bool,
                            normal: bool, program: &Program, uniforms: &U, params: &DrawParameters) where U: Uniforms {
        DrawBuffer::draw_once(ty, normal, texture, &self.display.clone(),
                              &mut self.target, program, uniforms, params, vertices
        )
    }

    pub fn get_text_size<T>(&self, text: T, params: &FontParameters) -> (f32, f32) where T: AsRef<str> {
        let fonts = self.fonts();
        let mut fonts = fonts.borrow_mut();
        fonts.get_string_bounds(text.as_ref(), params)
    }

    pub fn text<T>(&mut self, text: T, x: f32, y: f32, align_h: TextAlignHorizontal, align_v: TextAlignVertical, params: &FontParameters)
        where T: AsRef<str> {

        let text = text.as_ref();
        let viewport = self.viewport();
        let fonts = self.fonts().clone();
        let mut fonts = fonts.borrow_mut();
        let (w, h) = fonts.get_string_bounds(text, params);

        let x = match align_h {
            TextAlignHorizontal::Left => x,
            TextAlignHorizontal::Right => x - w,
            TextAlignHorizontal::Center => x - w / 2.0
        };
        let y = match align_v {
            TextAlignVertical::Top => y,
            TextAlignVertical::Bottom => y - h,
            TextAlignVertical::Center => y - h / 2.0
        };

        fonts.draw_string(&mut self.target, text, x, y, viewport, params);
    }

    pub fn into_inner(self) -> S {
        self.target
    }
}

pub enum TextAlignHorizontal {
    Left, Right, Center
}

pub enum TextAlignVertical {
    Top, Bottom, Center
}