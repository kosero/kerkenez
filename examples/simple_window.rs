use kerkenez::prelude::*;

fn main() {
    let mut app = App::new("Kerkenez Normal Reconstruction", 800, 600);

    // Add objects to visualize normals
    for i in 0..5 {
        app.draw(
            Cube::new()
                .at(i as f32 * 2.0 - 4.0, 0.0, 5.0 + i as f32)
                .rotate(0.5, i as f32, 0.0)
                .color(0.2, 0.5, 0.8, 1.0),
        );
    }

    // app.draw(
    //     Square::new()
    //         .at(0.0, -2.0, 10.0)
    //         .rotate(-1.57, 0.0, 0.0)
    //         .scale(20.0)
    //         .color(0.3, 0.3, 0.3, 1.0),
    // );

    let tex_mat = Material::new(
        "def",
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        Some("images/kerkenez.png"),
    );
    let mat_id = app.add_material(tex_mat);

    app.draw(
        Cube::new()
            .scale(5.0)
            .material(mat_id)
            .rotate(0.0, 45.0f32.to_radians(), 0.0)
            .at(0.0, 0.0, 1.0),
    );

    app.run();
}
