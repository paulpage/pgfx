use pgfx::app::{App};
use pgfx::types::{Rect, Color};
use pgfx::app::{Key, Scancode};

fn main() {
    let mut app = App::new("Text", "pgfx/res/DroidSans.ttf", 32.0);
    let background_color = Color::new(0, 100, 0);

    while !app.should_quit() {

        app.clear(background_color);

        if app.is_key_down(Key::A) {
            app.draw_rect(Rect::new(10.0, 10.0, 30.0, 30.0), Color::new(0, 0, 100));
        }
        if app.is_physical_key_pressed(Scancode::A) {
            app.draw_rect(Rect::new(100.0, 10.0, 30.0, 30.0), Color::new(0, 0, 100));
        }

        for s in &app.text_entered {
            println!("{}", s);
        }

        app.present();
    }
}
