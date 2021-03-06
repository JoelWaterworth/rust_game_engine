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
#[macro_use]
extern crate nom;

use std::collections::HashMap;
use winit::{Event, WindowEvent, ControlFlow};

mod camera;
mod renderer;
mod scene_object;
mod world;

use renderer::Renderer;

//TODO: implement shadows
//TODO: implement text

fn main() {
    Engine::run()
}

enum KeyState {
    Pressed,
    Released
}

struct Events {
    action: HashMap<String, KeyState>,
    axis: HashMap<String, f32>
}

impl Events {
    fn new() -> Events {
        Events {
            action: HashMap::new(),
            axis: HashMap::new(),
        }
    }
    pub fn init() -> Events {
        Events::new()
    }
}

pub struct Engine {
    renderer:   Renderer,
    window:     winit::Window,
}

impl Engine {
    fn init() -> (Self, winit::EventsLoop) {
        let events_loop = winit::EventsLoop::new();
        let monitor = events_loop.get_available_monitors().next();
        let program_name = "rustvulkantest";
        let engine_name = "rustvulkan";

        let window = winit::WindowBuilder::new()
            .with_title("Ash - Example")
            .with_fullscreen(monitor)
            .with_decorations(true)
            .build(&events_loop)
            .unwrap();

        let renderer = Renderer::init(engine_name, program_name, &window);
        (Engine {renderer, window}, events_loop)
    }

    pub fn run() {
        let (mut engine, mut event_loop) = Engine::init();
        engine.main_loop(&mut event_loop);
    }

    fn main_loop(&mut self, events_loop: &mut winit::EventsLoop) {
        events_loop.run_forever(|event| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Closed => return ControlFlow::Break,
                        WindowEvent::KeyboardInput {
                            input: winit::KeyboardInput { virtual_keycode: Some(winit::VirtualKeyCode::Escape), .. }, ..
                        } => return ControlFlow::Break,
                        _ => ()
                    }
                },
                _ => {}
            }
            self.renderer.render();
            ControlFlow::Continue
        });
    }
}
