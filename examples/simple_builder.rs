use pgfx::{rect, App, Engine, Texture, Key, Rect, Color, Point, Sound};

struct Example {
    texture: Texture,
}

impl App for Example {
    fn new(g: &mut Engine) -> Self {
        Self {
            texture: g.load_texture("res/textures/bird.png").unwrap(),
        }
    }

    fn update(&mut self, g: &mut Engine) {
        g.clear(Color::WHITE);
        g.draw_texture(
            &self.texture,
            rect!(0, 0, self.texture.width, self.texture.height),
            rect!(50, 50, 200, 200),
        );
    }
}

fn main() -> Result<(), String> {
    pgfx::app::<Example>("Builder Example").run()
}

