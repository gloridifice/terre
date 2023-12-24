use winit::event::KeyboardInput;
use crate::app::Runtime;
pub mod resource;

pub trait System{
    fn run(&self, runtime: &mut Runtime);
}

impl<F> System for F where F: Fn(&mut Runtime) -> (){
    fn run(&self, runtime: &mut Runtime) {
        self(runtime)
    }
}

pub trait KeyHandleSystem{
    fn run(&self, runtime: &mut Runtime, input: &KeyboardInput);
}

impl <F> KeyHandleSystem for F where F: Fn(&mut Runtime, &KeyboardInput) -> () {
    fn run(&self, runtime: &mut Runtime, input: &KeyboardInput) {
        self(runtime, input)
    }
}

pub enum Stage{
    Start, PreUpdate, Update, PostUpdate
}