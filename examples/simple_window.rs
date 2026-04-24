use kerkenez::prelude::*;

fn main() {
    let mut app = App::new("10k Squares", 800, 600);

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..10_000 {
        let x = rng.gen_range(-50.0..50.0);
        let y = rng.gen_range(-50.0..50.0);
        let z = rng.gen_range(0.0..100.0);

        app.draw(Square::new().at(x, y, z).scale(0.1).color(
            rng.r#gen::<f32>(),
            rng.r#gen::<f32>(),
            rng.r#gen::<f32>(),
        ));
    }

    app.run();
}
