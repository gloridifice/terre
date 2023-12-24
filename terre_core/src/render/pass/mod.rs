mod phong;

use hecs::World;
use crate::render::{FrameContext, RenderContext};

/// A queue of rendering pass.
/// # Usage
/// [`#new`](PassQueue::new) to create and [`#draw`](Pass::draw) to use it.
/// Use [`#push`](PassQueue::push) to push render pass.
pub struct PassQueue {
    //todo phase
    passes: Vec<Box<dyn Pass>>,
}

pub trait Pass {
    /// Provide `world` and `context`, but content that rendering need are
    /// updated into `context` per frame, so world is not usually needed.
    fn draw(&mut self, world: &World, context: &mut RenderContext, frame_context: &mut FrameContext);
}

impl PassQueue {
    pub fn new() -> Self{
        Self{
            passes: vec![]
        }
    }

    pub fn push(&mut self, pass: impl Pass + 'static){
        self.passes.push(Box::new(pass));
    }
}

impl Pass for PassQueue{
    fn draw(&mut self, world: &World, context: &mut RenderContext, frame_context: &mut FrameContext) {
        self.passes.iter_mut().for_each(|it|{
            it.draw(world, context, frame_context)
        });
    }
}