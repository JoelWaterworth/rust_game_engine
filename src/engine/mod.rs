use winit;
use self::renderer::*;

use std::collections::HashMap;

mod renderer;

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
    events:     Events
}

impl Engine {
    fn init() -> Engine {
        let program_name = "rustvulkantest";
        let engine_name = "rustvulkan";
        let width: u32 = 800;
        let height: u32 = 800;

        let window = winit::WindowBuilder::new()
            .with_title("Ash - Example")
            .with_dimensions(width, height)
            .with_decorations(true)
            .build()
            .unwrap();

        let renderer = Renderer::init(engine_name, program_name, &window);
        let events = Events::init();
        Engine {renderer: renderer, window: window, events: events}
    }

    pub fn run() {
        let mut engine = Engine::init();
        engine.main_loop();
    }

    fn main_loop(&mut self) {
        'render: loop {
            for event in self.window.poll_events() {
                match event {
                    winit::Event::KeyboardInput(_, _, Some(winit::VirtualKeyCode::Escape)) |
                    winit::Event::Closed => break 'render,
                    //winit::Event::KeyboardInput(winit::ElementState::Pressed, _, Some(winit::VirtualKeyCode::))
                    _ => (),
                }
                self.renderer.render();
            }
        }
    }
}
