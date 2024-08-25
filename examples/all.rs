use pgfx::{app, App, Engine, Texture, Key, Rect, Color, Point, Sound};

use std::time::{Duration, Instant};
use rand::Rng;

struct Example {
    rects: Vec<Rect>,
    colors: Vec<Color>,
    rotations: Vec<f32>,
    background_color: Color,
    scroll_offset: f32,
    tex_bird: Texture,
    pos: Point,
    drag_offset: Point,
    rotation: f32,
    last_mouse: Point,
    mouse_delta: Point,
    music: Sound,
    sound: Sound,
}

impl App for Example {
    fn new(g: &mut Engine) -> Self {

        let rect_count = 1000;
        let mut rng = rand::thread_rng();

        let mut rects = vec![Rect::new(0.0, 0.0, 0.0, 0.0); rect_count];
        for i in 0..rect_count {
            rects[i] = Rect::new(rng.gen_range(1..600) as f32, rng.gen_range(1..800) as f32, rng.gen_range(10..30) as f32, rng.gen_range(10..30) as f32);
        }


        let mut colors = vec![Color::BLACK; rect_count];
        for i in 0..rect_count {
            colors[i] = Color::new(rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255));
        }

        let rotations = vec![0.0; rect_count];

        let music = g.load_sound("res/music/sample.ogg");
        g.play_music(&music);

        Self {
            rects,
            colors,
            rotations,
            background_color: Color::new(0, 100, 0),
            scroll_offset: 0.0,
            tex_bird: g.load_texture("res/textures/bird.png").unwrap(),
            pos: Point::new(200.0, 200.0),
            drag_offset: Point::ZERO,
            rotation: 0.0,
            last_mouse: Point::ZERO,
            mouse_delta: Point::ZERO,
            music,
            sound: g.load_sound("res/sounds/tweet.ogg"),
        }
    }

    fn update(&mut self, g: &mut Engine) {
        let mut ui = g.ui();

        ui.show_demo_window(&mut true);

        self.mouse_delta = g.mouse - self.last_mouse;
        self.last_mouse = g.mouse;

        for i in 0..self.rects.len() {
            self.rotations[i] = g.mouse.x / 600.0;
        }

        self.pos = g.mouse;
        self.scroll_offset += g.scroll.y;

        g.clear(self.background_color);

        if g.mouse_left_down {
            self.rotation -= 0.05;
        }
        if g.mouse_right_down {
            self.rotation += 0.05;
        }
        if g.mouse_left_pressed || g.mouse_right_pressed {
            g.resume_music();
        }
        if !g.mouse_right_down && !g.mouse_left_down {
            g.pause_music();
        }

        if g.is_key_pressed(Key::Space) {
            g.play_sound(&self.sound);
            println!("space pressed");
        }

        for i in 0..self.rects.len() {
            g.draw_rotated_rect(self.rects[i], self.colors[i], Point::new(self.rects[i].width / 2.0, self.rects[i].height / 2.0), self.rotations[i]);
        }

        g.draw_rotated_texture(
            &self.tex_bird,
            Rect::new(0.0, 0.0, self.tex_bird.width, self.tex_bird.height),
            Rect::new(self.pos.x, self.pos.y, self.tex_bird.width * 4.0, self.tex_bird.height * 4.0),
            Point::new(self.tex_bird.width * 2.0, self.tex_bird.height * 2.0),
            self.rotation,
        );
        g.draw_text("Hello World!", 30.0, 30.0 + self.scroll_offset, 20.0, Color::new(0, 0, 100));
    }
}

fn main() {
    pgfx::app::<Example>("Rect example")
        .font("res/fonts/vera/Vera.ttf", 32.0)
        .with_ui()
        .run();
}
