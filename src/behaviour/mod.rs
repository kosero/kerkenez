use crate::input::keyboard::KeyboardState;

pub struct Context<'a> {
    pub keyboard: &'a KeyboardState,
}

pub trait Behaviour {
    fn init(&mut self, _ctx: &Context) {}
    fn update(&mut self, _ctx: &Context) {}
    fn exit(&mut self) {}
}
