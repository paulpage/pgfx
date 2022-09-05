use sdl2::event::{Event, WindowEvent};
use pgfx::canvas::Canvas;
use pgfx::types::{Rect, Color};

fn main() {
    let mut sdl = sdl2::init().unwrap();
    // TODO don't use hard coded font
    let mut canvas = Canvas::new(&mut sdl, "/usr/share/fonts/TTF/DejaVuSans.ttf", 32);

    let background_color = Color::new(0, 100, 0, 255);

    'mainloop: loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::Window { win_event, .. } => {
                    match win_event {
                        WindowEvent::Resized(width, height) => {
                            canvas.resize(width, height);
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        canvas.clear(background_color);

        canvas.draw_rect(Rect::new(5, 5, 20, 30), Color::new(100, 0, 0, 255));
        canvas.draw_text("Hello World!", 30, 30, 20.0, Color::new(0, 0, 100, 255));

        canvas.present();
    }
}
