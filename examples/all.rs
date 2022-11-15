use pgfx::canvas::{Canvas, Texture};
use pgfx::types::{Rect, Color, Point};
use std::time::{Duration, Instant};

fn main() {

    let mut app = Canvas::new("/usr/share/fonts/TTF/DejaVuSans.ttf", 32);
    let background_color = Color::new(0, 100, 0, 255);
    let mut scroll_offset = 0;
    let texture = Texture::from_file("/usr/share/icons/hicolor/128x128/apps/firefox.png").unwrap();
    
    let mut pos = Point::new(200, 200);
    let mut drag_offset = Point::new(0, 0);

    let mut rotation = 0.0;

    while !app.should_quit {


        app.update();

        // if app.mouse_middle_pressed {
        //     println!("Hello middle");
        //     drag_offset = app.mouse - pos;
        // }

        // if app.mouse_middle_down {
        //     pos = app.mouse - drag_offset;
        // }

        pos = app.mouse;

        scroll_offset += app.scroll.y;

        app.clear(background_color);

        if app.mouse_left_pressed {
            println!("hello left");
        }
        if app.mouse_left_down {
            app.draw_rect(Rect::new(10, 0, 10, 10), Color::new(0, 0, 100, 255));
        }
        if app.mouse_right_down {
            app.draw_rect(Rect::new(20, 0, 10, 10), Color::new(0, 0, 100, 255));
            rotation += 0.02;
        }
        if app.mouse_right_pressed {
            println!("hello right");

        }

        app.draw_rotated_rect(Rect::new(pos.x, pos.y, 200, 300), Color::new(100, 0, 0, 255), Point::new(100, 150), rotation);
        app.draw_text("Hello World!", 30, 30 + scroll_offset, 20.0, Color::new(0, 0, 100, 255));

        app.draw_texture(&texture, Rect::new(64, 64, 64, 64), Rect::new(5, 5, 128, 128));

        let start = Instant::now();
        app.present();
        let duration = start.elapsed();
        println!("present time: {:?}", duration);

    }
}
