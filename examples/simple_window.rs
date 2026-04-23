use kerkenez::{
    behaviour::{Behaviour, Context},
    input::Key,
    prelude::*,
};

struct Bebis;

impl Behaviour for Bebis {
    fn update(&mut self, ctx: &Context) {
        if ctx.keyboard.is_pressed(Key::Space) {
            println!("cik cik!");
        }
    }
}

fn main() {
    App::new("simple window", 800, 600)
        .with_behaviour(Bebis)
        .run();
}
