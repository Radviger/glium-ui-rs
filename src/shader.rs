use glium::program::Program;
use glium::Display;

use std::rc::Rc;
use std::collections::HashMap;

#[macro_export]
macro_rules! shader {
    ($display:expr, $name:literal) => {{
        use glium::program::Program;
        Program::from_source($display,
            &include_str!(concat!("resources/shaders/", $name, ".vsh")),
            &include_str!(concat!("resources/shaders/", $name, ".fsh")),
            None
        ).expect(concat!("Unable to compile `", $name, "` shader"))
    }};
}

pub struct ShaderManager {
    display: Display,
    programs: HashMap<String, Rc<Box<Program>>>
}

impl ShaderManager {
    pub fn new(display: &Display) -> ShaderManager {
        let mut programs = HashMap::new();
        programs.insert("font".into(), Rc::new(Box::new(
            shader!(display, "font")
        )));
        programs.insert("default".into(), Rc::new(Box::new(
            shader!(display, "default")
        )));
        programs.insert("textured".into(), Rc::new(Box::new(
            shader!(display, "textured")
        )));

        ShaderManager {
            display: display.clone(),
            programs
        }
    }

    pub fn font(&self) -> Rc<Box<Program>> {
        self.programs.get("font".into()).cloned().expect("Font shader is missing")
    }

    pub fn default(&self) -> Rc<Box<Program>> {
        self.programs.get("default".into()).cloned().expect("Default shader is missing")
    }

    pub fn textured(&self) -> Rc<Box<Program>> {
        self.programs.get("textured".into()).cloned().expect("Textured shader is missing")
    }
}