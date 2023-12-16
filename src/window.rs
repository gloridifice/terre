use cgmath::num_traits::clamp;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window;

pub struct AppWindow {
    pub event_loop: EventLoop<()>,
    pub window: window::Window,
}

impl AppWindow {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = window::WindowBuilder::new().with_title("Terre").build(&event_loop).unwrap();

        Self { event_loop, window }
    }

    pub fn run(event_loop: &EventLoop<()>, window: &mut window::Window, mut callback: impl 'static + FnMut(WindowEvents) -> ()) {
    }
}

pub enum WindowEvents<'a> {
    Resized {
        width: u32,
        height: u32,
    },
    Keyboard {
        state: &'a ElementState,
        virtual_keycode: &'a Option<VirtualKeyCode>,
    },
    Cursor{
        position: &'a PhysicalPosition<f64>
    },
    Draw,
}


