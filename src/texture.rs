use glium::texture::{Texture2d, RawImage2d, SrgbTexture2d};
use glium::Display;
use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;

use image::{self, ImageFormat, GenericImageView, ColorType};

pub struct TextureManager {
    pub display: Display,
    pub textures: HashMap<String, Rc<Box<SrgbTexture2d>>>
}

#[macro_export]
macro_rules! texture {
    ($manager:expr, $name:literal) => {{
        static IMAGE_BUF: &'static [u8] = include_bytes!(concat!("resources/", $name, ".png"));
        let manager = $manager;
        let image = crate::image::load_from_memory_with_format(&IMAGE_BUF, crate::image::ImageFormat::PNG)
            .expect("Image loading failed");
        let size = image.dimensions();
        let image = crate::glium::texture::RawImage2d::from_raw_rgba(image.raw_pixels(), size);
        let texture = crate::glium::texture::SrgbTexture2d::new(manager.display, image).expect("Texture allocation failed");
        manager.textures.insert($name.into(), std::rc::Rc::new(Box::new(texture)));
    }};
}

impl TextureManager {
    pub fn new(display: &Display) -> TextureManager {
        TextureManager {
            display: display.clone(),
            textures: HashMap::new()
        }
    }

    pub fn get<T>(&self, name: T) -> Rc<Box<SrgbTexture2d>> where T: AsRef<str> {
        self.textures.get(name.as_ref()).cloned().expect(&format!("Missing texture: {}", name.as_ref()))
    }

    pub fn get_or_load<P>(&mut self, name: String, path: P) -> Option<Rc<Box<SrgbTexture2d>>> where P: AsRef<Path> {
        if !self.textures.contains_key(&name) {
            let image = image::open(path.as_ref())
                .expect(&format!("Image loading failed: {}", name));

            let size = image.dimensions();
            let has_alpha = match image.color() {
                ColorType::Bgra8 => true,
                ColorType::La8 => true,
                ColorType::La16 => true,
                ColorType::Rgba8 => true,
                ColorType::Rgba16 => true,
                _ => false
            };
            let image: RawImage2d<u8> = if has_alpha {
                RawImage2d::from_raw_rgba(image.raw_pixels(), size)
            } else {
                RawImage2d::from_raw_rgb(image.raw_pixels(), size)
            };
            let texture = SrgbTexture2d::new(&self.display, image).expect("Texture allocation failed");
            self.textures.insert(name.clone(), Rc::new(Box::new(texture)));
        }
        self.textures.get(&name).cloned()
    }
}