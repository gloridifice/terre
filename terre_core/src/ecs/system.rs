use hecs::{Query, QueryMut, World};
use winit::event::KeyboardInput;
use std::marker::PhantomData;
use crate::app::State;
use crate::ecs::resource::ResManager;

pub trait System {
    fn run(&mut self, world: &mut World, res_manager: &mut ResManager);
}

pub trait KeyHandleSystem {
    fn run(&self, runtime: &mut State, input: &KeyboardInput);
}

impl<F> KeyHandleSystem for F where F: Fn(&mut State, &KeyboardInput) -> () {
    fn run(&self, runtime: &mut State, input: &KeyboardInput) {
        self(runtime, input)
    }
}

/// # Usage
/// System parameter passed into a function system need impl [`SystemParam`].
/// # Explanation
/// `impl System for FunctionSystem` use [`#get_param`](SystemParam::get_param) to get parameter from world.
pub trait SystemParam {
    type Item<'world>;
    fn get_param<'w>(world: &'w mut World, res_manager: &mut ResManager) -> Self::Item<'w>;
}

pub trait IntoSystem<Params> {
    type Output: System + 'static;
    fn into_system(self) -> Self::Output;
}

pub struct FunctionSystem<F, Marker> {
    system: F,
    marker: PhantomData<Marker>,
}

/// # Usage
/// The actual functions we pass into [`App#add_system`](App::add_system) need to impl [`SystemParamFunction`].
/// # Explanation
/// functions that implemented [`SystemParamFunction`] implemented [`IntoSystem`]. They will be turn into
/// [`FunctionSystem`]
pub trait SystemParamFunction<Marker>: 'static {
    type Params: SystemParam;

    /// Will be executed in ['System#run'](System::run)
    fn run<'w>(&mut self, param: <Self::Params as SystemParam>::Item<'w>);
}

//todo implement more and test
impl SystemParam for () {
    type Item<'world> = ();
    fn get_param<'w>(world: &'w mut World, res_manager: &mut ResManager) -> Self::Item<'w> {}
}
impl<P1> SystemParam for (P1, ) where P1: SystemParam {
    type Item<'world> = (P1::Item<'world>, );
    fn get_param<'w>(world: &'w mut World, res_manager: &mut ResManager) -> Self::Item<'w> {
        (P1::get_param(world, res_manager), )
    }
}

impl SystemParam for &mut World{
    type Item<'world> = &'world mut World;
    fn get_param<'w>(world: &'w mut World, res_manager: &mut ResManager) -> Self::Item<'w> {
        world
    }
}

impl<Func> SystemParamFunction<fn() -> ()> for Func where Func: FnMut() -> () + 'static {
    type Params = ();
    fn run<'w>(&mut self, param: <Self::Params as SystemParam>::Item<'w>) {
        self()
    }
}

impl<Func: 'static, P1> SystemParamFunction<fn(P1,) -> ()> for Func
    where Func: FnMut(P1,) -> () + FnMut(P1::Item<'_>) -> (),
          P1: SystemParam {
    type Params = (P1, );
    fn run<'w>(&mut self, param: <(P1, ) as SystemParam>::Item<'w>) {
        self(param.0)
    }
}


impl<F, Marker> IntoSystem<Marker> for F
    where
        Marker: 'static,
        F: SystemParamFunction<Marker> {
    type Output = FunctionSystem<F, Marker>;

    fn into_system(self) -> Self::Output {
        FunctionSystem {
            system: self,
            marker: PhantomData,
        }
    }
}

impl<F, Marker> System for FunctionSystem<F, Marker>
    where F: SystemParamFunction<Marker> + 'static {
    fn run(&mut self, world: &mut World, res_manager: &mut ResManager) {
        self.system.run(F::Params::get_param(world, res_manager));
    }
}

impl<Qy> SystemParam for QueryMut<'_, Qy> where Qy: Query{
    type Item<'world> = QueryMut<'world, Qy>;
    fn get_param<'w>(world: &'w mut World, res_manager: &mut ResManager) -> Self::Item<'w> {
        world.query_mut::<Qy>()
    }
}
