use pgfx::{rect, App, Engine, Texture, Key, Rect, Color, Point, Sound};

fn main() {
    let mut g = Engine::new("Example");

    let texture = g.load_texture("res/textures/bird.png").unwrap();

    while g.update() {
        g.clear(Color::WHITE);
        g.draw_texture(
            &texture,
            rect!(0, 0, texture.width, texture.height),
            rect!(50, 50, 200, 200),
        );
    }
}
