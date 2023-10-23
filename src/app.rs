use sdl2::Sdl;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdl2::video::{GLProfile, Window, GLContext};
use sdl2::keyboard::Mod;
use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;

use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, decoder::LoopedDecoder, OutputStream, OutputStreamHandle, Sink};
use rodio::source::{Buffered, Source};

use std::{ptr, mem};
use gl::types::*;
use stb_image::{self, image::LoadResult};
use cgmath::{Matrix4, Vector2, Deg, Vector3, Point3, SquareMatrix, Vector4};

use rusttype::{point, Font, Scale, PositionedGlyph};

use super::types::{Rect, Color, Point};
use super::opengl::{create_program, debug_callback};
// use super::keys::Key;

use enum_map::{enum_map, Enum, EnumMap};

use std::collections::HashSet;
pub type Scancode = sdl2::keyboard::Scancode;
pub type Key = sdl2::keyboard::Keycode;

#[derive(PartialEq)]
enum DrawType {
    Any,
    Triangles,
    Textures(u32),
    Models,
    Text,
}

pub struct App<'a> {
    // SDL
    pub sdl: Sdl,
    window: Window,
    gl_ctx: GLContext,

    // OpenGL
    program: u32,
    program_2d: u32,
    program_text: u32,
    program_texture: u32,
    uniforms: Uniforms,
    tri_buffer: u32,
    tri_vertices: Vec<f32>,
    last_tri_vertices_len: usize,
    tex_vertices: Vec<f32>,
    text_entries: Vec<(u32, Vec<f32>)>,
    last_draw_type: DrawType,

    // Window
    pub window_width: f32,
    pub window_height: f32,

    // Text
    pub char_width: f32,
    pub font_size: f32,
    pub font: Font<'a>,
    font_cache: HashMap<FontCacheKey, Rc<FontCacheEntry>>,

    // Input
    pub mouse: Point,
    pub scroll: Point,
    pub mouse_left_down: bool,
    pub mouse_left_pressed: bool,
    pub mouse_right_down: bool,
    pub mouse_right_pressed: bool,
    pub mouse_middle_down: bool,
    pub mouse_middle_pressed: bool,

    pub keys_down: HashSet<Key>,
    pub keys_pressed: HashSet<Key>,
    pub physical_keys_down: HashSet<Scancode>,
    pub physical_keys_pressed: HashSet<Scancode>,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
    pub shift_pressed: bool,
    pub text_entered: Vec<String>,

    // Audio
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sinks: Vec<Sink>,
    next_sink: usize,
}

impl<'a> App<'a> {

    pub fn new(title: &str, font_path: &str, font_size: f32) -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsys = sdl.video().unwrap();
        let gl_attr = video_subsys.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        let window = video_subsys
            .window(title, 800, 600)
            .position_centered()
            .resizable()
            .maximized()
            .opengl()
            .build()
            .unwrap();
        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsys.gl_get_proc_address(name) as *const _);

        gl::load_with(|ptr| video_subsys.gl_get_proc_address(ptr) as *const _);

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), ptr::null());
            gl::Enable(gl::BLEND);
        }

        let program = create_program(include_str!("shaders/3d.vert"), include_str!("shaders/3d.frag"));
        let program_2d = create_program(include_str!("shaders/2d.vert"), include_str!("shaders/2d.frag"));
        let program_text = create_program(include_str!("shaders/text.vert"), include_str!("shaders/text.frag"));
        let program_texture = create_program(include_str!("shaders/texture.vert"), include_str!("shaders/texture.frag"));

        let font = {
            let data = std::fs::read(Path::new(font_path)).unwrap();
            Font::try_from_vec(data).unwrap()
        };

        let char_width = font.glyph('o').scaled(Scale::uniform(font_size)).h_metrics().advance_width;

        let uniforms = unsafe {
            Uniforms {
                world: gl::GetUniformLocation(program, b"world\0".as_ptr() as *const _),
                view: gl::GetUniformLocation(program, b"view\0".as_ptr() as *const _),
                proj: gl::GetUniformLocation(program, b"proj\0".as_ptr() as *const _),
                view_position: gl::GetUniformLocation(program, b"view_position\0".as_ptr() as *const _),
                light_position: gl::GetUniformLocation(program, b"light.position\0".as_ptr() as *const _),
                light_direction: gl::GetUniformLocation(program, b"light.direction\0".as_ptr() as *const _),
                light_ambient: gl::GetUniformLocation(program, b"light.ambient\0".as_ptr() as *const _),
                light_diffuse: gl::GetUniformLocation(program, b"light.diffuse\0".as_ptr() as *const _),
                light_specular: gl::GetUniformLocation(program, b"light.specular\0".as_ptr() as *const _),
            }
        };
        let mut tri_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut tri_buffer);
        }

        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        let mut sinks = Vec::new();
        for i in 0..8 {
            sinks.push(Sink::try_new(&_stream_handle).unwrap());
        }

        Self {
            sdl,
            char_width,
            font_size: font_size,
            window_width: 800.0,
            window_height: 600.0,
            font,
            font_cache: HashMap::new(),
            window,
            gl_ctx,
            program,
            program_2d,
            program_text,
            program_texture,
            uniforms,
            mouse: Point::new(0.0, 0.0),
            scroll: Point::new(0.0, 0.0),
            mouse_left_down: false,
            mouse_left_pressed: false,
            mouse_right_down: false,
            mouse_right_pressed: false,
            mouse_middle_down: false,
            mouse_middle_pressed: false,
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            physical_keys_down: HashSet::new(),
            physical_keys_pressed: HashSet::new(),
            ctrl_pressed: false,
            alt_pressed: false,
            shift_pressed: false,
            text_entered: Vec::new(),
            tri_buffer,
            tri_vertices: Vec::new(),
            last_tri_vertices_len: 0,
            tex_vertices: Vec::new(),
            text_entries: Vec::new(),
            last_draw_type: DrawType::Any,
            _stream,
            _stream_handle,
            sinks,
            next_sink: 0,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    // pub fn set_active_region(&mut self, rect: Rect) {
    //     self.rect = rect;
    // }

    pub fn clear(&self, color: Color) {
        unsafe {
            gl::ClearColor(
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn present(&mut self) {
        self.flush();
        self.window.gl_swap_window();
    }

    fn flush(&mut self) {
        match self.last_draw_type {
            DrawType::Triangles => self.flush_triangles(),
            DrawType::Textures(texture_id) => self.flush_textures(texture_id),
            DrawType::Models => self.flush_models(),
            DrawType::Text => self.flush_text(),
            DrawType::Any => (),
        }
        self.last_draw_type = DrawType::Any;
    }

    fn process_batch(&mut self, target_type: DrawType) {
        if self.last_draw_type != target_type && self.last_draw_type != DrawType::Any {
            self.flush();
        }
        self.last_draw_type = target_type;
    }
}

// Input ============================================================

impl<'a> App<'a> {

    pub fn should_quit(&mut self) -> bool {

        let mut should_quit = false;

        self.scroll.x = 0.0;
        self.scroll.y = 0.0;
        self.mouse_left_pressed = false;
        self.mouse_right_pressed = false;
        self.mouse_middle_pressed = false;

        self.text_entered.clear();
        self.physical_keys_pressed.clear();
        self.keys_pressed.clear();

        for event in self.sdl.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => should_quit = true,
                Event::Window { win_event, .. } => {
                    match win_event {
                        WindowEvent::Resized(width, height) => self.resize(width as f32, height as f32),
                        _ => (),
                    }
                }
                Event::MouseWheel { x, y, .. } => {
                    self.scroll.x += x as f32 * 10.0;
                    self.scroll.y += y as f32 * 10.0;
                }
                Event::MouseMotion { x, y, .. } => {
                    self.mouse.x = x as f32;
                    self.mouse.y = y as f32;
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_left_down = false;
                        }
                        MouseButton::Right => {
                            self.mouse_right_down = false;
                        }
                        MouseButton::Middle => {
                            self.mouse_middle_down = false;
                        }
                        _ => ()
                    }
                }
                Event::MouseButtonDown { mouse_btn, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_left_pressed = true;
                            self.mouse_left_down = true;
                        }
                        MouseButton::Right => {
                            self.mouse_right_pressed = true;
                            self.mouse_right_down = true;
                        }
                        MouseButton::Middle => {
                            self.mouse_middle_pressed = true;
                            self.mouse_middle_down = true;
                        }
                        _ => ()
                    }
                }
                Event::KeyDown { keycode, scancode, keymod, .. } => {
                    if keymod.contains(Mod::RCTRLMOD) || keymod.contains(Mod::LCTRLMOD) {
                        self.ctrl_pressed = true;
                    }
                    if keymod.contains(Mod::RALTMOD) || keymod.contains(Mod::LALTMOD) {
                        self.alt_pressed = true;
                    }
                    if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                        self.shift_pressed = true;
                    }
                    if let Some(scancode) = scancode {
                        self.physical_keys_down.insert(scancode);
                        self.physical_keys_pressed.insert(scancode);
                    }
                    if let Some(keycode) = keycode {
                        self.keys_down.insert(keycode);
                        self.keys_pressed.insert(keycode);
                    }
                }
                Event::KeyUp { keycode, scancode, keymod, .. } => {
                    if !(keymod.contains(Mod::RCTRLMOD) || keymod.contains(Mod::LCTRLMOD)) {
                        self.ctrl_pressed = false;
                    }
                    if !(keymod.contains(Mod::RALTMOD) || keymod.contains(Mod::LALTMOD)) {
                        self.alt_pressed = false;
                    }
                    if !(keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD)) {
                        self.shift_pressed = false;
                    }
                    if let Some(scancode) = scancode {
                        self.physical_keys_down.remove(&scancode);
                    }
                    if let Some(keycode) = keycode {
                        self.keys_down.remove(&keycode);
                    }
                }
                Event::TextInput { text, .. } => {
                    self.text_entered.push(text);
                }

                _ => (),
            }
        }
        // TODO

        should_quit
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_physical_key_down(&self, scancode: Scancode) -> bool {
        self.physical_keys_down.contains(&scancode)
    }

    pub fn is_physical_key_pressed(&self, scancode: Scancode) -> bool {
        self.physical_keys_pressed.contains(&scancode)
    }

    pub fn get_key_string(&self, key: &Key) -> String {
        let mut kstr = String::new();
        if self.ctrl_pressed {
            kstr.push_str("c-");
        }
        if self.alt_pressed {
            kstr.push_str("a-");
        }
        if self.shift_pressed {
            kstr.push_str("s-");
        }
        kstr.push_str(&key.to_string().to_ascii_lowercase());
        kstr
    }
}

// Shapes ============================================================

fn get_rect_vertices(rect: Rect, origin: Point, rotation: f32, window_width: f32, window_height: f32) -> [f32; 8] {
    let x = rect.x;
    let y = rect.y;
    let width = rect.width;
    let height =  rect.height;
    let dx = -origin.x;
    let dy = -origin.y;

    let (x1, y1, x2, y2, x3, y3, x4, y4) = if rotation == 0.0 {
        let x = x + dx;
        let y = y + dy;
        (
            x, y,
            x + width, y,
            x, y + height,
            x + width, y + height,
        )
    } else {
        let rcos = rotation.cos();
        let rsin = rotation.sin();
        (
            x + dx*rcos - dy*rsin,
            y + dx*rsin + dy*rcos,
            x + (dx + width)*rcos - dy*rsin,
            y + (dx + width)*rsin + dy*rcos,
            x + dx*rcos - (dy + height)*rsin,
            y + dx*rsin + (dy + height)*rcos,
            x + (dx + width)*rcos - (dy + height)*rsin,
            y + (dx + width)*rsin + (dy + height)*rcos,
        )
    };

    [
        x1 * 2.0 / window_width as f32 - 1.0,
        x2 * 2.0 / window_width as f32 - 1.0,
        x3 * 2.0 / window_width as f32 - 1.0,
        x4 * 2.0 / window_width as f32 - 1.0,
        1.0 - y1 * 2.0 / window_height as f32,
        1.0 - y2 * 2.0 / window_height as f32,
        1.0 - y3 * 2.0 / window_height as f32,
        1.0 - y4 * 2.0 / window_height as f32,
    ]
}

impl<'a> App<'a> {

    fn flush_triangles(&mut self) {
        let mut vao_2d = 0;
        unsafe {
            gl::Disable(gl::DEPTH_TEST);

            gl::GenVertexArrays(1, &mut vao_2d);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.tri_buffer);
            if self.tri_vertices.len() == self.last_tri_vertices_len {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (self.tri_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    self.tri_vertices.as_ptr() as *const _,
                );
            } else {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.tri_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    self.tri_vertices.as_ptr() as *const _,
                    gl::DYNAMIC_DRAW
                );
            }

            gl::BindVertexArray(vao_2d);
            let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);

            gl::UseProgram(self.program_2d);

            gl::DrawArrays(gl::TRIANGLES, 0, self.tri_vertices.len() as GLsizei / 6);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            gl::DeleteVertexArrays(1, &mut vao_2d);
        }

        self.last_tri_vertices_len = self.tri_vertices.len();
        self.tri_vertices.clear();
    }

    pub fn draw_rotated_rect(&mut self, rect: Rect, color: Color, origin: Point, rotation: f32) {

        let [x1, x2, x3, x4, y1, y2, y3, y4] = get_rect_vertices(rect, origin, rotation, self.window_width, self.window_height);

        self.tri_vertices.extend_from_slice(&[
            x1, y1, color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, color.a as f32 / 255.0,
            x2, y2, color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, color.a as f32 / 255.0,
            x4, y4, color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, color.a as f32 / 255.0,
            x1, y1, color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, color.a as f32 / 255.0,
            x4, y4, color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, color.a as f32 / 255.0,
            x3, y3, color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, color.a as f32 / 255.0,
        ]);

        self.process_batch(DrawType::Triangles);
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.draw_rotated_rect(rect, color, Point::new(0.0, 0.0), 0.0);
    }
}


// Textures ============================================================

pub struct Texture {
    pub width: f32,
    pub height: f32,
    pub data: Vec<u8>,
    texture_id: u32,
}

impl Texture {

    pub fn new(width: usize, height: usize, data: Vec<u8>) -> Self {
        // Load the texture from the buffer
        let texture_id = unsafe {
            let mut texture_id: u32 = 0;
            gl::ActiveTexture(gl::TEXTURE0);
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width as GLint,
                height as GLint,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );

            texture_id
        };

        Texture {
            width: width as f32,
            height: height as f32,
            data,
            texture_id,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        match stb_image::image::load(path) {
            LoadResult::ImageU8(image) => Ok(Self::new(image.width, image.height, image.data)),
            _ => Err("Failed to load texture".to_string()),
        }
    }
}

impl<'a> App<'a> {

    pub fn flush_textures(&mut self, texture_id: u32) {
        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);

            // TODO Decide what these should be.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.tex_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                self.tex_vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            gl::BindVertexArray(vao);
            let stride = 4 * mem::size_of::<GLfloat>() as GLsizei;


            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);

            let uniform = gl::GetUniformLocation(self.program_texture, b"tex\0".as_ptr() as *const _);
            gl::UseProgram(self.program_texture);
            gl::Uniform1i(uniform, 0);

            gl::DrawArrays(gl::TRIANGLES, 0, self.tex_vertices.len() as GLsizei / 4);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        unsafe {
            gl::DeleteBuffers(1, &mut vbo);
            gl::DeleteVertexArrays(1, &mut vao);
            // gl::DeleteTextures(1, &mut id);
        }
        self.tex_vertices.clear();
    }

    pub fn draw_rotated_texture(&mut self, texture: &Texture, src_rect: Rect, dest_rect: Rect, origin: Point, rotation: f32) {
        self.process_batch(DrawType::Textures(texture.texture_id));

        let [x1, x2, x3, x4, y1, y2, y3, y4] = get_rect_vertices(dest_rect, origin, rotation, self.window_width, self.window_height);

        let u0 = src_rect.x / texture.width;
        let u1 = (src_rect.x + src_rect.width) / texture.width;
        let v0 = (src_rect.y + src_rect.height) / texture.height;
        let v1 = src_rect.y / texture.height;

        let new_vertices = [
            x1, y1, u0, v1,
            x2, y2, u1, v1,
            x4, y4, u1, v0,
            x1, y1, u0, v1,
            x4, y4, u1, v0,
            x3, y3, u0, v0,
        ];
        self.tex_vertices.extend_from_slice(&new_vertices);
    }

    pub fn draw_texture(&mut self, texture: &Texture, src_rect: Rect, dest_rect: Rect) {
        self.draw_rotated_texture(texture, src_rect, dest_rect, Point::new(0.0, 0.0), 0.0);
    }
}

// Text ============================================================

// TODO does this need size included?
#[derive(Hash, PartialEq)]
struct FontCacheKey {
    c: String,
    color: Color,
}

struct FontCacheEntry {
    texture_id: u32,
    width: i32,
    height: i32,
}

impl Eq for FontCacheKey {}


impl<'a> App<'a> {

    pub fn flush_text(&mut self) {
        for (id, vertices) in &self.text_entries {
            let (mut vao, mut vbo) = (0, 0);
            unsafe {

                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, *id);

                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::Disable(gl::DEPTH_TEST);


                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    vertices.as_ptr() as *const _,
                    gl::STATIC_DRAW
                );
                gl::BindVertexArray(vao);
                let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;


                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (4 * mem::size_of::<GLfloat>()) as *const _);

                gl::UseProgram(self.program_text);
                let uniform = gl::GetUniformLocation(self.program_text, b"tex\0".as_ptr() as *const _);
                gl::Uniform1i(uniform, 0);

                gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei / 8);

                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::BindVertexArray(0);
            }

            unsafe {
                gl::DeleteBuffers(1, &mut vbo);
                gl::DeleteVertexArrays(1, &mut vao);
                // gl::DeleteTextures(1, id);
                // gl::DeleteProgram(program);
            }
        }
        self.text_entries.clear();
    }

    pub fn layout_text(&self, text: &str, scale: f32) -> (Vec<PositionedGlyph<'_>>, usize, usize) {
        let font_scale = Scale::uniform(scale);
        let v_metrics = self.font.v_metrics(font_scale);
        let glyphs: Vec<_> = self.font
            .layout(text, font_scale, point(0.0, 0.0 + v_metrics.ascent))
            .collect();

        let height = (v_metrics.ascent - v_metrics.descent).ceil() as usize;
        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;
        (glyphs, width, height)
    }

    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: Color) -> Rect {
        // Save the original parameters to return in the rect
        let input_x = x;
        let input_y = y;

        let key = FontCacheKey {
            c: text.to_string(),
            color,
        };

        let tex = self.font_cache.get(&key).cloned().unwrap_or_else(|| {
            let (glyphs, glyphs_width, glyphs_height) = self.layout_text(text, scale);
            let mut buffer: Vec<f32> = vec![0.0; glyphs_width * glyphs_height];
            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {

                    let min_x = bounding_box.min.x;
                    let min_y = bounding_box.min.y;

                    glyph.draw(|x, y, v| {
                        let x = std::cmp::max(x as i32 + min_x, 1) as usize - 1;
                        let y = std::cmp::max(y as i32 + min_y, 1) as usize - 1;
                        let index = y * glyphs_width + x;
                        buffer[index] = v;
                    });
                }
            }

            // Load the texture from the buffer
            let id = unsafe {
                let mut id: u32 = 0;
                gl::ActiveTexture(gl::TEXTURE0);
                gl::GenTextures(1, &mut id);
                gl::BindTexture(gl::TEXTURE_2D, id);

                // TODO Decide what these should be.
                // TODO should these be by the draw or the load?
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as GLint,
                    glyphs_width as GLint,
                    glyphs_height as GLint,
                    0,
                    gl::RED,
                    gl::FLOAT,
                    buffer.as_ptr() as *const _
                );
                id
            };
            let resource = Rc::new(FontCacheEntry {
                texture_id: id,
                width: glyphs_width as i32,
                height: glyphs_height as i32,
            });
            self.font_cache.insert(key, resource.clone());
            resource
        });

        let id = tex.texture_id;
        let glyphs_width = tex.width;
        let glyphs_height = tex.height;

        let x = x * 2.0 / self.window_width - 1.0;
        let y = 1.0 - y * 2.0 / self.window_height;
        let height = glyphs_height as f32 * 2.0 / self.window_height;
        let width = glyphs_width as f32 * 2.0 / self.window_width;
        let y = y - height;
        let color = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];
        let vertices = [
            x, y, 0.0, 1.0, color[0], color[1], color[2], color[3],
            x + width, y, 1.0, 1.0, color[0], color[1], color[2], color[3],
            x + width, y + height, 1.0, 0.0, color[0], color[1], color[2], color[3],
            x, y, 0.0, 1.0, color[0], color[1], color[2], color[3],
            x + width, y + height, 1.0, 0.0, color[0], color[1], color[2], color[3],
            x, y + height, 0.0, 0.0, color[0], color[1], color[2], color[3],
        ];

        self.text_entries.push((id, Vec::from(vertices)));

        self.process_batch(DrawType::Text);

        Rect::new(input_x, input_y, glyphs_width as f32, glyphs_height as f32)
    }

    pub fn text_length(&self, text: &str) -> f32 {
        // TODO make this correct for non-monospace
        text.len() as f32 * self.char_width
        // let mut length = 0;
        // for c in text.chars() {
        //     let (x, _) = self.font.size_of_char(c).unwrap();
        //     length += x as i32;
        // }
        // length
    }

    pub fn set_font(&mut self, path: &Path, size: f32) {
        self.font = {
            let data = std::fs::read(path).unwrap();
            Font::try_from_vec(data).unwrap()
        };
        self.font_size = size;
    }
}

// Models ============================================================

pub struct Camera {
    pub focus: Point3<f32>,
    pub distance: f32,
    pub rot_horizontal: f32,
    pub rot_vertical: f32,
    pub fovy: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            focus: Point3::new(0.0, 0.0, 0.0),
            distance: 10.0,
            rot_horizontal: 0.5,
            rot_vertical: 0.5,
            fovy: 45.0,
        }
    }

    pub fn rotate(&mut self, horizontal: f32, vertical: f32) {
        self.rot_horizontal += horizontal;
        self.rot_vertical += vertical;
        if self.rot_vertical < 0.001 {
            self.rot_vertical = 0.001;
        }
        if self.rot_vertical > std::f32::consts::PI {
            self.rot_vertical = std::f32::consts::PI - 0.001;
        }
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::new(
            self.focus.z + self.distance * self.rot_vertical.sin() * self.rot_horizontal.sin(),
            self.focus.y + self.distance * self.rot_vertical.cos(),
            self.focus.x + self.distance * self.rot_vertical.sin() * self.rot_horizontal.cos()
        )
    }
}

pub struct Model {
    pub vao: u32,
    pub vertex_buffer_length: i32,
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub transform: Matrix4<f32>,
    pub position_offset: Vector3<f32>,
    pub rotation_offset: Vector3<f32>,
    pub bounding_box: BoundingBox,
}

impl Model {
    pub fn set_transform(&mut self) {
        let position = Vector3::new(self.position.x * 0.5, self.position.y * 0.2, self.position.z * 0.5);
        self.transform = Matrix4::from_translation(position - self.position_offset)
            * Matrix4::from_angle_x(Deg((self.rotation.x * 90.0) - self.rotation_offset.x))
            * Matrix4::from_angle_y(Deg((self.rotation.y * 90.0) - self.rotation_offset.y))
            * Matrix4::from_angle_z(Deg((self.rotation.z * 90.0) - self.rotation_offset.z))
    }
}

pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

pub struct Uniforms {
    world: GLint,
    view: GLint,
    proj: GLint,
    view_position: GLint,
    light_position: GLint,
    light_direction: GLint,
    light_ambient: GLint,
    light_diffuse: GLint,
    light_specular: GLint,
}

fn unproject(source: Vector3<f32>, view: Matrix4<f32>, proj: Matrix4<f32>) -> Vector3<f32> {
    let view_proj = (proj * view).invert().unwrap();
    let q = view_proj * Vector4::new(source.x, source.y, source.z, 1.0);
    Vector3::new(q.x / q.w, q.y / q.w, q.z / q.w)
}

fn get_mouse_ray(aspect_ratio: f32, mouse_position: Vector2<f32>, camera: &Camera) -> (Point3<f32>, Vector3<f32>) {
    let view = Matrix4::look_at_rh(camera.position(), camera.focus, Vector3::new(0.0, 1.0, 0.0));
    let proj = cgmath::perspective(Deg(camera.fovy), aspect_ratio, 0.01, 100.0);
    let near = unproject(Vector3::new(mouse_position.x, mouse_position.y, 0.0), view, proj);
    let far = unproject(Vector3::new(mouse_position.x, mouse_position.y, 1.0), view, proj);
    let direction = far - near;
    (camera.position(), direction)
}

impl<'a> App<'a> {

    pub fn flush_models(&mut self) {
        // TODO
    }

    pub fn start_3d(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn load_model(&mut self, vertices: &[f32]) -> (u32, i32) {
        // TODO there is no "unload_model" right now because this is meant to be run once for each
        // model, and all the memory can be cleaned up when the program exits.
        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            gl::BindVertexArray(vao);
            let stride = 10 * mem::size_of::<GLfloat>() as GLsizei;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const _);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        (vao, vertices.len() as i32)
    }


    pub fn draw_model(&self, vao: GLuint, vertex_buffer_length: i32, world: [f32; 16], view: [f32; 16], proj: [f32; 16], view_position: [f32; 3], light: [f32; 15]) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            // gl::UseProgram(self.program);

            gl::UniformMatrix4fv(self.uniforms.world, 1, gl::FALSE, world.as_ptr());
            gl::UniformMatrix4fv(self.uniforms.view, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.uniforms.proj, 1, gl::FALSE, proj.as_ptr());
            gl::Uniform3f(self.uniforms.view_position, view_position[0], view_position[1], view_position[2]);
            gl::Uniform3f(self.uniforms.light_position, light[0], light[1], light[2]);
            gl::Uniform3f(self.uniforms.light_direction, light[3], light[4], light[5]);
            gl::Uniform3f(self.uniforms.light_ambient, light[6], light[7], light[8]);
            gl::Uniform3f(self.uniforms.light_diffuse, light[9], light[10], light[11]);
            gl::Uniform3f(self.uniforms.light_specular, light[12], light[13], light[14]);

            gl::BindVertexArray(vao);
            // TODO make sure this is passing in the vertex count, not byte or float count
            gl::DrawArrays(gl::TRIANGLES, 0, vertex_buffer_length as GLsizei);
            // gl::BindVertexArray(0);
            gl::Disable(gl::DEPTH_TEST);
        }
    }

    // pub fn draw_bounding_box(&self, a: [f32; 3], b: [f32; 3], world: [f32; 16], view: [f32; 16], proj: [f32; 16], view_position: [f32; 3], light: [f32; 15]) {
    //     let c = vec![0.0, 1.0, 1.0, 0.3];
    //     let vertices = vec![
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3]
    //     ];
    //     self.draw_model(&vertices, world, view, proj, view_position, light);
    // }

}

// Audio ============================================================

pub type Sound = Buffered<LoopedDecoder<BufReader<File>>>;

impl<'a> App<'a> {
    pub fn load_sound(&mut self, path: &str) -> Sound {
        let f = BufReader::new(File::open(path).unwrap());
        Decoder::new_looped(f).unwrap().buffered()
    }

    pub fn play_music(&mut self, sound: &Sound) {
        self.sinks[0].clear();
        self.sinks[0].append(sound.clone().repeat_infinite());
    }

    pub fn pause_music(&mut self) {
        self.sinks[0].pause();
    }

    pub fn resume_music(&mut self) {
        self.sinks[0].play();
    }

    pub fn play_sound(&mut self, sound: &Sound) {
        // TODO detect free sinks
        let sink_idx = self.next_sink;
        self.next_sink += 1;
        if self.next_sink == 8 {
            self.next_sink = 1;
        }
        self.sinks[sink_idx].append(sound.clone());
    }
}
