use pgfx::canvas::Canvas;
use pgfx::types::{Rect, Color};

fn main() {

    let mut app = Canvas::new("/usr/share/fonts/TTF/DejaVuSans.ttf", 32);
    let background_color = Color::new(0, 100, 0, 255);
    let mut scroll_offset = 0;

    while !app.should_quit {
        app.update();

        scroll_offset += app.scroll.y;

        app.clear(background_color);

        app.draw_rect(Rect::new(5, 5 + scroll_offset, 20, 30), Color::new(100, 0, 0, 255));
        app.draw_text("Hello World!", 30, 30 + scroll_offset, 20.0, Color::new(0, 0, 100, 255));

        app.present();
    }
}
