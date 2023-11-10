use pgfx::app::{App, Texture};
use pgfx::types::{Rect, Color, Point};

fn main() {
    let mut app = App::new("Rect example", "res/DroidSans.ttf", 32.0);
    let background_color = Color::new(0, 0, 100);
    let mut rect = Rect::new(0.0, 0.0, 30.0, 60.0);
    let mut rect_color = Color::new(0, 100, 0);

    // let texture = Texture::from_file("res/pic.png").unwrap();
    // let tex_src_rect = Rect::new(0.0, 0.0, texture.width, texture.height);
    // let tex_dst_rect = Rect::new(0.0, 0.0, texture.width * 4.0, texture.height * 4.0);
    let mut rotation = 0.0;

    while !app.should_quit() {

        rotation += 0.01;

        rect.x = app.mouse.x;
        rect.y = app.mouse.y;

        app.clear(background_color);

        app.draw_rotated_rect(rect, rect_color, Point::ZERO, rotation);
        // app.draw_text("Hello World!", 30.0, 30.0, 20.0, Color::new(0, 0, 100));
        // app.draw_rotated_texture(&texture, tex_src_rect, tex_dst_rect, Point::ZERO, rotation);
        // app.draw_rotated_texture(&texture, tex_src_rect, tex_dst_rect, Point::ZERO, rotation);

        app.present();
    }
}
