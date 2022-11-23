use pgfx::app::{App, Texture};
use pgfx::types::{Rect, Color, Point};
use std::time::{Duration, Instant};
use rand::Rng;

fn main() {

    let rect_count = 1_000_000;

    let mut rng = rand::thread_rng();
    let mut rects = vec![Rect::new(0, 0, 0, 0); rect_count];
    for i in 0..rect_count {
        rects[i] = Rect::new(rng.gen_range(1..600), rng.gen_range(1..800), rng.gen_range(2..6), rng.gen_range(2..6));
    }

    let mut colors = vec![Color::new(0, 0, 0, 0); rect_count];
    for i in 0..rect_count {
        colors[i] = Color::new(rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255), 255);
    }

    let mut rotations = vec![0.0; rect_count];

    let mut app = App::new("/usr/share/fonts/TTF/DejaVuSans.ttf", 32);
    let background_color = Color::new(0, 100, 0, 255);
    let mut scroll_offset = 0;
    let texture = Texture::from_file("/usr/share/icons/hicolor/128x128/apps/firefox.png").unwrap();
    
    let mut pos = Point::new(200, 200);
    let mut drag_offset = Point::new(0, 0);

    let mut rotation = 0.0;

    let mut last_mouse = Point::new(0, 0);
    let mut mouse_delta = Point::new(0, 0);

    while !app.should_quit {
let very_start = Instant::now();
let start = Instant::now();

        mouse_delta = app.mouse - last_mouse;
        last_mouse = app.mouse;

        for i in 0..rect_count {
            // rects[i].x += rng.gen_range(-10..=10);
            // rects[i].y += rng.gen_range(-10..=10);
            // rotations[i] += (rng.gen_range(-3..=3) as f32 / 10.0);
            // rotations[i] += 0.03;
            rotations[i] = app.mouse.x as f32 / 600.0;
            // rects[i].width += (mouse_delta.x / 10) as u32;
            // rects[i].height += (mouse_delta.x / 10) as u32;
        }

        let start = Instant::now();

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

        app.draw_rects(&rects, &colors, &rotations);
        // for i in 0..rect_count {
        //     app.draw_rect(rects[i], colors[i]);
        // }

        app.draw_rotated_rect(Rect::new(pos.x, pos.y, 200, 300), Color::new(100, 0, 0, 255), Point::new(100, 150), rotation);
        app.draw_text("Hello World!", 30, 30 + scroll_offset, 20.0, Color::new(0, 0, 100, 255));

        app.draw_texture(&texture, Rect::new(64, 64, 64, 64), Rect::new(5, 5, 128, 128));

        app.present();
    }
}
