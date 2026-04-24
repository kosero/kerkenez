use kerkenez::prelude::*;

fn main() {
    let mut app = App::new("Street", 1280, 720);

    app.set_ambient_light(0.1, 0.1, 0.1, 0.1);
    app.set_directional_light(
        DirectionalLight::new()
            .direction(0.5, -0.8, -0.2)
            .intensity(0.3),
    );

    app.run(|app, _dt| {
        // Road
        app.draw(
            DrawCommand::cube()
                .at(0.0, -2.0, -10.0)
                .scale_xyz(12.0, 0.1, 80.0)
                .tint(0.15, 0.15, 0.15, 1.0),
        );

        // Road markings (dashed line)
        for i in 0..20 {
            let z = 5.0 - (i as f32) * 4.0;
            app.draw(
                DrawCommand::cube()
                    .at(0.0, -1.94, z)
                    .scale_xyz(0.2, 0.05, 2.0)
                    .tint(0.9, 0.9, 0.9, 1.0),
            );
        }

        // Sidewalks
        app.draw(
            DrawCommand::cube()
                .at(-7.5, -1.9, -10.0)
                .scale_xyz(3.0, 0.2, 80.0)
                .tint(0.3, 0.3, 0.3, 1.0),
        );
        app.draw(
            DrawCommand::cube()
                .at(7.5, -1.9, -10.0)
                .scale_xyz(3.0, 0.2, 80.0)
                .tint(0.3, 0.3, 0.3, 1.0),
        );

        // Buildings
        let building_colors = [
            (0.6, 0.3, 0.2), // Brick red
            (0.8, 0.8, 0.7), // Beige
            (0.3, 0.4, 0.5), // Blueish
            (0.7, 0.6, 0.5), // Brown
            (0.9, 0.9, 0.9), // White
        ];

        for side in 0..2 {
            let sign = if side == 0 { -1.0 } else { 1.0 };
            let x_pos = sign * 11.5;

            for i in 0..12 {
                let z = 5.0 - (i as f32) * 6.0;
                let color_idx = (i + side) % building_colors.len();
                let (r, g, b) = building_colors[color_idx];

                // Randomize height a bit
                let height = 10.0 + ((i * 3) % 5) as f32 * 2.5;

                // Building main block
                app.draw(
                    DrawCommand::cube()
                        .at(x_pos, -1.8 + height / 2.0, z)
                        .scale_xyz(5.0, height, 5.0)
                        .tint(r, g, b, 1.0),
                );

                // Windows
                let floors = (height / 2.5) as i32;
                for f in 1..floors {
                    let y = -1.8 + (f as f32) * 2.5;
                    for w in 0..2 {
                        let wx = x_pos - sign * 2.5; // slightly protruding to show glass
                        let wz = z - 1.5 + (w as f32) * 3.0;

                        // Window glass
                        app.draw(
                            DrawCommand::cube()
                                .at(wx, y, wz)
                                .scale_xyz(0.1, 1.2, 1.0)
                                .tint(0.5, 0.8, 1.0, 1.0), // Light blue glass
                        );

                        // Balcony (only some floors)
                        if (f + i as i32) % 2 == 0 {
                            app.draw(
                                DrawCommand::cube()
                                    .at(wx - sign * 0.6, y - 0.6, wz)
                                    .scale_xyz(1.4, 0.2, 1.6)
                                    .tint(0.2, 0.2, 0.2, 1.0),
                            );
                        }
                    }
                }

                // Shop at ground floor
                app.draw(
                    DrawCommand::cube()
                        .at(x_pos - sign * 2.5, -0.5, z)
                        .scale_xyz(0.2, 2.0, 4.0)
                        .tint(0.05, 0.05, 0.05, 1.0), // Dark glass front
                );
            }
        }

        // A car
        // Body
        app.draw(
            DrawCommand::cube()
                .at(2.5, -1.3, -2.0)
                .scale_xyz(2.0, 0.8, 4.2)
                .tint(0.8, 0.1, 0.1, 1.0), // Red car
        );
        // Cabin
        app.draw(
            DrawCommand::cube()
                .at(2.5, -0.5, -2.2)
                .scale_xyz(1.8, 0.8, 2.0)
                .tint(0.1, 0.1, 0.1, 1.0), // Black windows
        );
        // Wheels
        for wx in [-1.0, 1.0].iter() {
            for wz in [-1.5, 1.5].iter() {
                app.draw(
                    DrawCommand::cube()
                        .at(2.5 + wx * 1.0, -1.6, -2.0 + wz)
                        .scale_xyz(0.4, 0.6, 0.6)
                        .tint(0.05, 0.05, 0.05, 1.0),
                );
            }
        }

        // A truck on the other side
        app.draw(
            DrawCommand::cube()
                .at(-2.5, -0.5, -12.0)
                .scale_xyz(2.5, 2.5, 6.0)
                .tint(0.2, 0.4, 0.8, 1.0), // Blue truck
        );
        app.draw(
            DrawCommand::cube()
                .at(-2.5, -0.8, -8.0)
                .scale_xyz(2.5, 1.8, 2.0)
                .tint(0.9, 0.9, 0.9, 1.0), // White cabin
        );

        // Street lights
        for i in 0..6 {
            let z = 2.0 - (i as f32) * 12.0;
            for side in [-1.0, 1.0].iter() {
                let x = side * 5.5;
                // Pole
                app.draw(
                    DrawCommand::cube()
                        .at(x, 1.5, z)
                        .scale_xyz(0.2, 7.0, 0.2)
                        .tint(0.2, 0.2, 0.2, 1.0),
                );
                // Arm
                app.draw(
                    DrawCommand::cube()
                        .at(x - side * 1.0, 4.9, z)
                        .scale_xyz(2.0, 0.15, 0.2)
                        .tint(0.2, 0.2, 0.2, 1.0),
                );
                // Lamp
                app.draw(
                    DrawCommand::cube()
                        .at(x - side * 1.8, 4.8, z)
                        .scale_xyz(0.4, 0.1, 0.4)
                        .tint(1.0, 1.0, 0.6, 1.0), // Yellow light
                );
            }
        }

        // Some trees (cubist style) on the sidewalk
        for i in 0..8 {
            let z = 0.0 - (i as f32) * 9.0;
            for side in [-1.0, 1.0].iter() {
                let x = side * 7.5;
                // Trunk
                app.draw(
                    DrawCommand::cube()
                        .at(x, -1.0, z)
                        .scale_xyz(0.3, 2.0, 0.3)
                        .tint(0.4, 0.2, 0.1, 1.0),
                );
                // Leaves
                app.draw(
                    DrawCommand::cube()
                        .at(x, 0.5, z)
                        .scale_xyz(1.5, 1.5, 1.5)
                        .tint(0.2, 0.6, 0.2, 1.0),
                );
                app.draw(
                    DrawCommand::cube()
                        .at(x, 1.5, z)
                        .scale_xyz(1.0, 1.0, 1.0)
                        .tint(0.3, 0.7, 0.3, 1.0),
                );
            }
        }
    });
}
