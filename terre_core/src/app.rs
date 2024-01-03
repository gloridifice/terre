use winit::window::WindowBuilder;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use pollster::block_on;
use crate::ecs::resource::ResManager;
use crate::ecs::system::IntoSystem;
use crate::render::RenderState;
use crate::schedule::{GameSchedule, Stage};

pub struct App {
    world: hecs::World,
    res_manager: ResManager,
    schedule: GameSchedule,
}

impl App {
    pub fn new() -> Self {
        App {
            schedule: GameSchedule::new(),
            world: hecs::World::new(),
            res_manager: ResManager::new(),
        }
    }

    pub fn add_system<Params>(mut self, stage: Stage, function: impl IntoSystem<Params>) -> Self {
        self.schedule.add_system(stage, function);
        self
    }
    pub fn add_plugin(self, plugin: impl Plugin + 'static) -> Self {
        plugin.build(self)
    }

    pub fn run(mut self) {
        env_logger::init();

        let event_loop = EventLoop::new();

        let mut state = block_on(RenderState::new(
            WindowBuilder::new()
                .with_title("Terre")
                .build(&event_loop)
                .unwrap(),
        ));

        //run all starts system
        self.schedule.run_starts(&mut self.world, &mut self.res_manager);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window.id() => {
                    // let egui_renderer = runtime.res_manager.get_res_mut::<EguiRenderer>();
                    // egui_renderer.unwrap().handle_event(event);
                    match event {
                        WindowEvent::KeyboardInput {
                            input,
                            ..
                        } => {
                            // self.key_board_handles.iter().for_each(|it| { it.run(&mut state, input) });
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            // state.input.cursor_position = vec2(position.x, position.y);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            // runtime.resize(physical_size.clone())
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            // runtime.resize((*new_inner_size).clone())
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                    // run logic
                    self.schedule.run_updates(&mut self.world, &mut self.res_manager);
                    // run render todo split logic and render
                    state.render_context.render_and_present(&mut self.world, &mut state.pass_queue);
                }
                Event::RedrawEventsCleared => {
                    state.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}

pub trait Plugin {
    fn build(&self, app: App) -> App;
}

#[cfg(test)]
mod test{
    use hecs::World;
    use crate::app::App;
    use crate::schedule::Stage;

    #[test]
    fn test_start_schedule(){
        fn insert_one(world: &mut World){
            world.spawn((12i32,));
        }
        let mut app = App::new().add_system(Stage::Start, insert_one);

        app.schedule.run_starts(&mut app.world, &mut app.res_manager);

        let mut query = app.world.query::<&i32>();

        for (_id, a) in query.iter(){
            assert_eq!(a.clone(), 12i32);
        }
    }
}