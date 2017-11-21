#![allow(dead_code)]
#[macro_use]
extern crate ash;
extern crate winit;
#[cfg(windows)]
extern crate user32;
#[cfg(windows)]
extern crate winapi;

extern crate libc;
extern crate glsl_to_spirv;
extern crate cgmath;
extern crate tobj;
extern crate image;

mod engine;
mod camera;

use engine::Engine;

//TODO: implement shadows
//TODO: implement text

fn main() {
    Engine::run()
}
