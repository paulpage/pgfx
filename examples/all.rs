use pgfx::app::{App, Texture, Sound};
use pgfx::types::{Rect, Color, Point};
use std::time::{Duration, Instant};
use sdl2::mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
use rand::Rng;

fn main() {

    let mut rect_count = 1_000;

    let mut rng = rand::thread_rng();
    let mut rects = vec![Rect::new(0.0, 0.0, 0.0, 0.0); rect_count];
    for i in 0..rect_count {
        rects[i] = Rect::new(rng.gen_range(1..600) as f32, rng.gen_range(1..800) as f32, rng.gen_range(10..30) as f32, rng.gen_range(10..30) as f32);
    }

    let mut colors = vec![Color::BLACK; rect_count];
    for i in 0..rect_count {
        colors[i] = Color::new(rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255));
    }

    let mut rotations = vec![0.0; rect_count];

    let mut app = App::new("PGFX Example", "/usr/share/fonts/TTF/DejaVuSansMono.ttf", 32.0);
    let background_color = Color::new(0, 100, 0);
    let mut scroll_offset = 0.0;
    let rat = Texture::from_file("rat2.png").unwrap();
    let texture = Texture::from_file("/usr/share/icons/hicolor/128x128/apps/firefox.png").unwrap();
    
    let mut pos = Point::new(200.0, 200.0);
    let mut drag_offset = Point::ZERO;

    let mut rotation = 0.0;

    let mut last_mouse = Point::ZERO;
    let mut mouse_delta = Point::ZERO;

    let music = app.load_music("spinning_rat.ogg");
    // let music_backwards = app.load_sound("tar_gninnips.ogg");
    // let sound = app.load_sound("/home/paul/pop.ogg");
    // let bark = app.load_sound("/home/paul/bark.ogg");
    app.play_music();
    // music_backwards.play_loop();

    let s1 = app.load_sound("spinning_rat.ogg");
    let s2 = app.load_sound("spinning_rat.ogg");
    let s3 = app.load_sound("spinning_rat.ogg");
    let s4 = app.load_sound("spinning_rat.ogg");

    let mut force_allocation = true;
    let mut alloc_count = 1000;
    while !app.should_quit {
let start = Instant::now();

        mouse_delta = app.mouse - last_mouse;
        last_mouse = app.mouse;

        for i in 0..rect_count {
            // rects[i].x += rng.gen_range(-10..=10);
            // rects[i].y += rng.gen_range(-10..=10);
            // rotations[i] += (rng.gen_range(-3..=3) as f32 / 10.0);
            // rotations[i] += 0.03;
            rotations[i] = app.mouse.x / 600.0;
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

        if app.mouse_left_down {
            app.draw_rect(Rect::new(10.0, 0.0, 10.0, 10.0), Color::new(0, 0, 100));
            rotation -= 0.002;
        }
        if app.mouse_right_down {
            app.draw_rect(Rect::new(20.0, 0.0, 10.0, 10.0), Color::new(0, 0, 100));
            rotation += 0.002;
        }

        if app.mouse_right_pressed {
            // music_backwards.resume();
        }
        if app.mouse_left_pressed {
            app.resume_music();
        }
        // if app.mouse_right_pressed || app.mouse_left_pressed {
        //     music.resume();
        // }
        if !app.mouse_right_down && !app.mouse_left_down {
            app.pause_music();
            // music_backwards.pause();
        }

        // if app.mouse_middle_pressed {
        //     bark.play();
        // }

        // for i in 0..rect_count {
        //     app.draw_rotated_rect(rects[i], colors[i], Point::new(rects[i].width as i32 / 2, rects[i].height as i32 / 2), rotations[i]);
        // }
        // for i in 0..rect_count {
        //     app.draw_text("Hello, World!", rects[i].x, rects[i].y, 20.0, Color::new(0, 0, 100));
        // }
        for i in 0..rect_count {
            app.draw_rotated_texture(&texture, Rect::new(texture.width / 2.0, texture.height / 2.0, texture.width / 2.0, texture.height / 2.0), rects[i], Point::new(rects[i].width / 2.0, rects[i].height / 2.0), rotations[i]);
        }

        // app.draw_rotated_rect(Rect::new(pos.x, pos.y, 200, 300), Color::new(100, 0, 0), Point::new(100, 150), rotation);
        app.draw_rotated_texture(&rat, Rect::new(0.0, 0.0, rat.width, rat.height), Rect::new(pos.x, pos.y, rat.width * 4.0, rat.height * 4.0), Point::new(rat.width * 2.0, rat.height * 2.0), rotation);
        app.draw_text("Hello World!", 30.0, 30.0 + scroll_offset, 20.0, Color::new(0, 0, 100));

        app.draw_texture(&texture, Rect::new(texture.width / 2.0, texture.height / 2.0, texture.width / 2.0, texture.height / 2.0), Rect::new(5.0, 5.0, 128.0, 128.0));
        app.draw_texture(&texture, Rect::new(0.0, 0.0, texture.width, texture.height), Rect::new(200.0, 200.0, 128.0, 128.0));

        app.present();
        // println!("Frame time: {:?}", Instant::now() - start);
    }
}
