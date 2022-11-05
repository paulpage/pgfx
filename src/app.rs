use super::types::Point

pub struct App {
    sdl: 
    pub mouse: Point,
    pub scroll: Point,
    pub mouse_left_down: bool,
    pub mouse_left_pressed: bool,
    pub mouse_right_down: bool,
    pub mouse_right_pressed: bool,
}

impl App {
    pub fn new() -> Self {

        Self {
            mouse: Point::new(0, 0),
            scroll: Point::new(0, 0),
            mouse_left_down: false,
            mouse_left_pressed: false,
            mouse_right_down: false,
            mouse_right_pressed: false,
        }
    }
}
