// use pgfx::app::{App};
// use pgfx::types::{Rect, Color};
// use pgfx::app::{Key, Scancode};

use pgfx::app;

#[macroquad::main("Test")]
async fn main() {

    let mut pos_y = 0.0;
    let font = app::load_ttf_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf").await.unwrap();

    loop {
        app::clear_background(app::RED);

        app::draw_line(40.0, 40.0, 100.0, 200.0, 15.0, app::BLUE);
        app::draw_rectangle(app::screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, app::GREEN);
        app::draw_circle(app::screen_width() - 30.0, app::screen_height() - 30.0, 15.0, app::YELLOW);

        let (_wheel_x, wheel_y) = app::mouse_wheel();
        pos_y += wheel_y * 30.0;

        app::draw_text_ex("Hello!", 20.0, 20.0 + pos_y, app::TextParams {
            font: Some(&font),
            font_size: 30,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            rotation: 0.0,
            color: app::BLUE,
        });

        app::next_frame().await
    }



    // let mut app = App::new("Text", "pgfx/res/DroidSans.ttf", 32.0);
    // let background_color = Color::new(0, 100, 0);

    // while !app.should_quit() {

    //     app.clear(background_color);

    //     if app.is_key_down(Key::A) {
    //         app.draw_rect(Rect::new(10.0, 10.0, 30.0, 30.0), Color::new(0, 0, 100));
    //     }
    //     if app.is_physical_key_pressed(Scancode::A) {
    //         app.draw_rect(Rect::new(100.0, 10.0, 30.0, 30.0), Color::new(0, 0, 100));
    //     }

    //     for s in &app.text_entered {
    //         println!("{}", s);
    //     }

    //     for key in &app.keys_pressed {
    //         println!("{}", app.get_key_string(key));
    //     }

    //     app.present();
    // }
}
