use sdl2::{Sdl, EventPump};
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdl2::video::{GLProfile, Window, GLContext, SwapInterval};
use sdl2::keyboard::Mod;
use std::collections::HashMap;
use std::rc::Rc;
use std::path::{Path, PathBuf};

use std::{ptr, mem};
use gl::types::*;
use stb_image::{self, image::LoadResult};

use rusttype::{point, Font, Scale, PositionedGlyph};

use super::types::{Rect, Color, Point};
use super::opengl::{create_program, debug_callback};
use super::imgui::Imgui;
use super::sound::{SoundEngine, Sound};
use std::collections::HashSet;

pub type Scancode = sdl2::keyboard::Scancode;
pub type Key = sdl2::keyboard::Keycode;

// Builder ============================================================

pub trait App {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine);
}

pub struct AppBuilder<T: App> {
    title: String,
    font_path: Option<String>,
    font_size: f32,
    enable_ui: bool,
    resource_path: PathBuf,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: App> AppBuilder<T> {
    pub fn font(mut self, path: &str, size: f32) -> Self {
        self.font_path = Some(path.to_string());
        self.font_size = size;
        self
    }

    pub fn with_ui(mut self) -> Self {
        self.enable_ui = true;
        self
    }

    pub fn resource_path(mut self, path: &str) -> Self {
        self.resource_path = PathBuf::from(path);
        self
    }

    pub fn run(self) -> Result<(), String> {
        let mut engine = Engine::new(&self.title, self.font_path, self.font_size, &self.resource_path);
        let mut app = T::new(&mut engine);
        loop {
            let mut event_pump = engine.sdl.event_pump().unwrap();
            engine.ui.prepare_frame(&engine.window, &event_pump);
            app.update(&mut engine);
            if engine.should_quit(&mut event_pump) {
                break;
            }
        }
        Ok(())
    }
}

pub fn app<T: App>(title: &str) -> AppBuilder<T> {
    AppBuilder {
        title: title.to_string(),
        font_path: None,
        font_size: 16.0,
        enable_ui: false,
        resource_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        _phantom: std::marker::PhantomData,
    }
}

// Struct ============================================================

#[derive(PartialEq)]
#[allow(dead_code)]
enum DrawType {
    Any,
    Triangles,
    Textures(u32),
    Text,
}

pub struct Engine<'a> {
    // SDL
    pub sdl: Sdl,
    window: Window,
    _gl_ctx: GLContext,

    // OpenGL
    program_2d: u32,
    program_text: u32,
    program_texture: u32,
    tri_buffer: u32,
    text_buffer: u32,
    tri_vertices: Vec<f32>,
    last_tri_vertices_len: usize,
    tex_vertices: Vec<f32>,
    text_entries: Vec<(u32, Vec<f32>)>,
    last_draw_type: DrawType,

    // Window
    pub window_width: f32,
    pub window_height: f32,
    pub window_size_changed: bool,

    // Events
    pub has_events: bool,
    quit_requested: bool,

    // Text
    pub char_width: f32,
    pub font_size: f32,
    pub font: Font<'a>,
    font_cache: HashMap<String, Rc<FontCacheEntry>>,

    // Input
    pub mouse: Point,
    pub scroll: Point,
    pub mouse_left_down: bool,
    pub mouse_left_pressed: bool,
    pub mouse_left_clicks: u8,
    pub mouse_right_down: bool,
    pub mouse_right_pressed: bool,
    pub mouse_right_clicks: u8,
    pub mouse_middle_down: bool,
    pub mouse_middle_pressed: bool,
    pub mouse_middle_clicks: u8,

    pub keys_down: HashSet<Key>,
    pub keys_pressed: HashSet<Key>,
    pub physical_keys_down: HashSet<Scancode>,
    pub physical_keys_pressed: HashSet<Scancode>,
    pub ctrl_down: bool,
    pub alt_down: bool,
    pub shift_down: bool,
    pub text_entered: Vec<String>,

    // Subsystems
    pub sound: SoundEngine,
    pub ui: Imgui,

    draw_ui_this_frame: bool,
    resource_path: PathBuf,
}

impl<'a> Engine<'a> {

    pub fn new(title: &str, font_path: Option<String>, font_size: f32, resource_path: &Path) -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsys = sdl.video().unwrap();
        let gl_attr = video_subsys.gl_attr();
        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_profile(GLProfile::Core);
        let window = video_subsys
            .window(title, 800, 600)
            .position_centered()
            .resizable()
            .maximized()
            .opengl()
            .build()
            .unwrap();
        let _gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|ptr| video_subsys.gl_get_proc_address(ptr) as *const _);
        window.gl_make_current(&_gl_ctx).unwrap();

        // video_subsys.gl_set_swap_interval(SwapInterval::Immediate);

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), ptr::null());
            gl::Enable(gl::BLEND);
        }

        let program_2d = create_program(include_str!("shaders/2d.vert"), include_str!("shaders/2d.frag"));
        let program_text = create_program(include_str!("shaders/text.vert"), include_str!("shaders/text.frag"));
        let program_texture = create_program(include_str!("shaders/texture.vert"), include_str!("shaders/texture.frag"));

        let font = {
            let data = if let Some(path) = font_path {
                std::fs::read(Path::new(&path)).unwrap()
            } else {
                include_bytes!("../res/fonts/vera/Vera.ttf").to_vec()
            };
            Font::try_from_vec(data).unwrap()
        };

        let char_width = font.glyph('o').scaled(Scale::uniform(font_size)).h_metrics().advance_width;

        let mut tri_buffer = 0;
        let mut text_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut tri_buffer);
            gl::GenBuffers(1, &mut text_buffer);

            let init_text_vertices = [0.0; 48];
            gl::BindBuffer(gl::ARRAY_BUFFER, text_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (init_text_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                init_text_vertices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
        }

        let ui = Imgui::new(&window);
        let sound = SoundEngine::new();

        Self {
            sdl,
            char_width,
            font_size,
            window_width: 800.0,
            window_height: 600.0,
            window_size_changed: false,
            font,
            font_cache: HashMap::new(),
            window,
            _gl_ctx,
            program_2d,
            program_text,
            program_texture,
            has_events: true,
            quit_requested: false,
            mouse: Point::new(0.0, 0.0),
            scroll: Point::new(0.0, 0.0),
            mouse_left_down: false,
            mouse_left_pressed: false,
            mouse_left_clicks: 0,
            mouse_right_down: false,
            mouse_right_pressed: false,
            mouse_right_clicks: 0,
            mouse_middle_down: false,
            mouse_middle_pressed: false,
            mouse_middle_clicks: 0,
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            physical_keys_down: HashSet::new(),
            physical_keys_pressed: HashSet::new(),
            ctrl_down: false,
            alt_down: false,
            shift_down: false,
            text_entered: Vec::new(),
            tri_buffer,
            text_buffer,
            tri_vertices: Vec::new(),
            last_tri_vertices_len: 0,
            tex_vertices: Vec::new(),
            text_entries: Vec::new(),
            last_draw_type: DrawType::Any,
            sound,
            ui,
            draw_ui_this_frame: false,
            resource_path: PathBuf::from(resource_path),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.window_size_changed = true;
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

    fn flush(&mut self) {
        match self.last_draw_type {
            DrawType::Triangles => self.flush_triangles(),
            DrawType::Textures(texture_id) => self.flush_textures(texture_id),
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

impl<'a> Engine<'a> {

    pub fn should_quit(&mut self, event_pump: &mut EventPump) -> bool {

        if self.draw_ui_this_frame {
            self.ui.render();
            self.draw_ui_this_frame = false;
        }

        self.flush();
        self.window.gl_swap_window();

        // ========================================

        let mut should_quit = self.quit_requested;
        self.has_events = false;
        self.window_size_changed = false;

        self.scroll.x = 0.0;
        self.scroll.y = 0.0;
        self.mouse_left_pressed = false;
        self.mouse_right_pressed = false;
        self.mouse_middle_pressed = false;
        self.mouse_left_clicks = 0;
        self.mouse_right_clicks = 0;
        self.mouse_middle_clicks = 0;

        self.text_entered.clear();
        self.physical_keys_pressed.clear();
        self.keys_pressed.clear();

        for event in event_pump.poll_iter() {

            self.ui.handle_event(&event);

            self.has_events = true;
            match event {
                Event::Quit { .. } => should_quit = true,
                Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
                    self.resize(width as f32, height as f32);
                }
                Event::MouseWheel { precise_x, precise_y, .. } => {
                    self.scroll.x += precise_x as f32;
                    self.scroll.y += precise_y as f32;
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
                Event::MouseButtonDown { mouse_btn, clicks, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_left_pressed = true;
                            self.mouse_left_down = true;
                            self.mouse_left_clicks = clicks;
                        }
                        MouseButton::Right => {
                            self.mouse_right_pressed = true;
                            self.mouse_right_down = true;
                            self.mouse_right_clicks = clicks;
                        }
                        MouseButton::Middle => {
                            self.mouse_middle_pressed = true;
                            self.mouse_middle_down = true;
                            self.mouse_middle_clicks = clicks;
                        }
                        _ => ()
                    }
                }
                Event::MultiGesture { x, y, .. } => {
                    println!("multigesture {x} {y}");
                }
                Event::KeyDown { keycode, scancode, keymod, .. } => {
                    if keymod.contains(Mod::RCTRLMOD) || keymod.contains(Mod::LCTRLMOD) {
                        self.ctrl_down = true;
                    }
                    if keymod.contains(Mod::RALTMOD) || keymod.contains(Mod::LALTMOD) {
                        self.alt_down = true;
                    }
                    if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                        self.shift_down = true;
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
                        self.ctrl_down = false;
                    }
                    if !(keymod.contains(Mod::RALTMOD) || keymod.contains(Mod::LALTMOD)) {
                        self.alt_down = false;
                    }
                    if !(keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD)) {
                        self.shift_down = false;
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

    pub fn quit(&mut self) {
        self.quit_requested = true;
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
        if self.ctrl_down {
            kstr.push_str("c-");
        }
        if self.alt_down {
            kstr.push_str("a-");
        }
        if self.shift_down {
            kstr.push_str("s-");
        }
        kstr.push_str(&key.to_string().to_ascii_lowercase());
        kstr
    }

    // UI

    pub fn ui(&mut self) -> &mut imgui::Ui {
        self.draw_ui_this_frame = true;
        self.ui.new_frame()
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
        x1 * 2.0 / window_width - 1.0,
        x2 * 2.0 / window_width - 1.0,
        x3 * 2.0 / window_width - 1.0,
        x4 * 2.0 / window_width - 1.0,
        1.0 - y1 * 2.0 / window_height,
        1.0 - y2 * 2.0 / window_height,
        1.0 - y3 * 2.0 / window_height,
        1.0 - y4 * 2.0 / window_height,
    ]
}

impl<'a> Engine<'a> {

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

            gl::DeleteVertexArrays(1, &vao_2d);
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

impl<'a> Engine<'a> {

    pub fn res_path(&self, path: &str) -> String {
        self.resource_path.join(path).to_str().expect("Invalid UTF-8 in path").to_string()
    }

    pub fn load_texture(&self, path: &str) -> Result<Texture, String> {
        Texture::from_file(&self.res_path(path))
    }

    pub fn load_sound(&mut self, path: &str) -> Sound {
        self.sound.load(&self.res_path(path))
    }

    pub fn play_sound(&mut self, sound: &Sound) {
        self.sound.play(sound)
    }

    pub fn play_music(&mut self, sound: &Sound) {
        self.sound.play_music(sound)
    }

    pub fn pause_music(&mut self) {
        self.sound.pause_music()
    }

    pub fn resume_music(&mut self) {
        self.sound.resume_music()
    }


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
            gl::DeleteBuffers(1, &vbo);
            gl::DeleteVertexArrays(1, &vao);
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

struct FontCacheEntry {
    texture_id: u32,
    width: i32,
    height: i32,
}

impl<'a> Engine<'a> {

    pub fn flush_text(&mut self) {

        let mut vao = 0;
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);

            gl::UseProgram(self.program_text);
            let uniform = gl::GetUniformLocation(self.program_text, b"tex\0".as_ptr() as *const _);
            gl::Uniform1i(uniform, 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.text_buffer);
        }

        for (id, vertices) in &self.text_entries {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, *id);

                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    vertices.as_ptr() as *const _,
                );

                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);
                let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (4 * mem::size_of::<GLfloat>()) as *const _);

                gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei / 8);

                gl::BindVertexArray(0);
                gl::DeleteVertexArrays(1, &vao);
            }
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
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
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;
        (glyphs, width, height)
    }

    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: Color) -> Rect {
        // Save the original parameters to return in the rect
        let input_x = x;
        let input_y = y;

        let key = text.to_string();

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

    pub fn set_resource_path(&mut self, path: &str) {
        self.resource_path = PathBuf::from(path);
    }

}
