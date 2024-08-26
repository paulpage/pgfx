use std::ops;

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }

    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
    };
}

impl ops::Add<Self> for Point {
    type Output = Self;
    fn add(self, _rhs: Self) -> Self {
        Point { x: self.x + _rhs.x, y: self.y + _rhs.y }
    }
}

impl ops::Sub<Self> for Point {
    type Output = Self;
    fn sub(self, _rhs: Self) -> Self {
        Point { x: self.x - _rhs.x, y: self.y - _rhs.y }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[macro_export]
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as f32, $y as f32, $w as f32, $h as f32)
    )
);

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn union(&self, other: Rect) -> Rect {
        if self.width == 0.0 || self.height == 0.0 {
            return other;
        }
        if other.width == 0.0 || other.height == 0.0 {
            return *self;
        }
        Rect {
            x: f32::min(self.x, other.x),
            y: f32::min(self.y, other.y),
            width: f32::max(self.x + self.width, other.x + other.width) - f32::min(self.x, other.x),
            height: f32::max(self.y + self.height, other.y + other.height) - f32::min(self.y, other.y),
        }
    }

    pub fn has_intersection(&self, other: Rect) -> bool {
        self.x <= other.x + other.width && self.x + self.width >= other.x
         && self.y <= other.y + other.height && self.y + self.height >= other.y
    }

    pub fn intersection(&self, other: Rect) -> Rect {
        if !Rect::has_intersection(self, other) {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }
        Rect {
            x: f32::max(self.x, other.x),
            y: f32::max(self.y, other.y),
            width: f32::min(self.x + self.width, other.x + other.width) - f32::max(self.x, other.x),
            height: f32::min(self.y + self.height, other.y + other.height) - f32::max(self.y, other.y),
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(color_hex_str: &str) -> Result<Self, std::num::ParseIntError> {
        let color = i32::from_str_radix(color_hex_str, 16)?;
        let b = color % 0x100;
        let g = (color - b) / 0x100 % 0x100;
        let r = (color - g) / 0x10000;

        Ok(Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: 255,
        })
    }

    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const GRAY: Self = Self {
        r: 150,
        g: 150,
        b: 150,
        a: 255,
    };

}
