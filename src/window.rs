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

    pub fn run(self, mut callback: impl 'static + FnMut(WindowEvents) -> ()) {
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    match event {
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput{
                                state,
                                virtual_keycode,
                                ..
                            },
                            ..
                        } => callback(WindowEvents::Keyboard {state, virtual_keycode}),
                        WindowEvent::CursorMoved {
                            position,
                            ..
                        } => {
                            callback(WindowEvents::Cursor {position})
                        },
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => callback(WindowEvents::Resized {
                            width: physical_size.width,
                            height: physical_size.height,
                        }),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            callback(WindowEvents::Resized {
                                width: new_inner_size.width,
                                height: new_inner_size.height,
                            })
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    callback(WindowEvents::Draw);
                    // match state.render() {
                    //     Ok(_) => {}
                    //     // Reconfigure the surface if it's lost or outdated
                    //     Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    //         // state.resize(state.size)
                    //     }
                    //     // The system is out of memory, we should probably quit
                    //     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    //     Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    // }
                }
                Event::RedrawEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
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


