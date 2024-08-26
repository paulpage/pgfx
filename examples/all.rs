use pgfx::{Engine, Texture, Key, Rect, Color, Point, Sound};

use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let mut g = Engine::new("Example");

    let background_color = Color::new(0, 100, 0);
    let rect_count = 1000;

    let mut rects = vec![Rect::new(0.0, 0.0, 0.0, 0.0); rect_count];
    for i in 0..rect_count {
        rects[i] = Rect::new(rng.gen_range(1..600) as f32, rng.gen_range(1..800) as f32, rng.gen_range(10..30) as f32, rng.gen_range(10..30) as f32);
    }

    let mut colors = vec![Color::BLACK; rect_count];
    for i in 0..rect_count {
        colors[i] = Color::new(rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255));
    }

    let mut rotations = vec![0.0; rect_count];

    let music = g.load_sound("res/music/sample.ogg");
    let tex_bird = g.load_texture("res/textures/bird.png").unwrap();
    let sound = g.load_sound("res/sounds/tweet.ogg");

    // State
    let mut scroll_offset = 0.0;
    //let mut drag_offset = Point::ZERO;
    let mut rotation = 0.0;
    //let mut last_mouse = Point::ZERO;
    //let mut mouse_delta = Point::ZERO;

    g.play_music(&music);

    while g.update() {
        let ui = g.ui();

        ui.show_demo_window(&mut true);

        //mouse_delta = g.mouse - last_mouse;
        //last_mouse = g.mouse;

        for i in 0..rects.len() {
            rotations[i] = g.mouse.x / 600.0;
        }

        scroll_offset += g.scroll.y;

        g.clear(background_color);

        if g.mouse_left_down {
            rotation -= 0.05;
        }
        if g.mouse_right_down {
            rotation += 0.05;
        }
        if g.mouse_left_pressed || g.mouse_right_pressed {
            g.resume_music();
        }
        if !g.mouse_right_down && !g.mouse_left_down {
            g.pause_music();
        }

        if g.is_key_pressed(Key::Space) {
            g.play_sound(&sound);
            println!("space pressed");
        }

        if g.is_key_pressed(Key::Q) {
            g.quit();
        }

        for i in 0..rects.len() {
            g.draw_rotated_rect(rects[i], colors[i], Point::new(rects[i].width / 2.0, rects[i].height / 2.0), rotations[i]);
        }

        g.draw_rotated_texture(
            &tex_bird,
            Rect::new(0.0, 0.0, tex_bird.width, tex_bird.height),
            Rect::new(g.mouse.x, g.mouse.y, tex_bird.width * 4.0, tex_bird.height * 4.0),
            Point::new(tex_bird.width * 2.0, tex_bird.height * 2.0),
            rotation,
        );
        g.draw_text("Hello World!", 30.0, 30.0 + scroll_offset * 10.0, 20.0, Color::new(0, 0, 100));
    }
}
