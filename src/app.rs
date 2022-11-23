use sdl2::Sdl;
use sdl2::event::{Event, WindowEvent};
use sdl2::render::{Texture as SdlTexture, WindowCanvas};
use sdl2::mouse::MouseButton;
use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;
use std::ffi::CString;
use std::{ptr, mem};
use cgmath::{Matrix4, Vector2, Deg, Vector3, Point3, SquareMatrix, Vector4};
use gl::types::*;
use stb_image::{self, image::LoadResult};
use std::time::{Duration, Instant};

use rusttype::{point, Font, Scale, PositionedGlyph};

use super::types::{Rect, Color, Point};

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn from_file(path: &str) -> Result<Texture, String> {
        let result = stb_image::image::load(path);
        if let LoadResult::ImageU8(image) = result {
            return Ok(Texture {
                width: image.width,
                height: image.height,
                data: image.data,
            });
        };
        Err("Failed to load texture".to_string())
    }
}

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
    pub position: Vector3<i32>,
    pub rotation: Vector3<i32>,
    pub transform: Matrix4<f32>,
    pub position_offset: Vector3<f32>,
    pub rotation_offset: Vector3<f32>,
    pub bounding_box: BoundingBox,
}

impl Model {
    pub fn set_transform(&mut self) {
        let position = Vector3::new(self.position.x as f32 * 0.5, self.position.y as f32 * 0.2, self.position.z as f32 * 0.5);
        self.transform = Matrix4::from_translation(position - self.position_offset)
            * Matrix4::from_angle_x(Deg((self.rotation.x * 90) as f32 - self.rotation_offset.x))
            * Matrix4::from_angle_y(Deg((self.rotation.y * 90) as f32 - self.rotation_offset.y))
            * Matrix4::from_angle_z(Deg((self.rotation.z * 90) as f32 - self.rotation_offset.z))
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

fn create_shader(shader_type: u32, source: &str) -> u32 {
    unsafe {
        let id = gl::CreateShader(shader_type);
        let source_cstr = CString::new(source).unwrap();
        gl::ShaderSource(
            id,
            1,
            &source_cstr.as_ptr(),
            std::ptr::null()
        );
        gl::CompileShader(id);
        let mut success: gl::types::GLint = 1;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            let error = {
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));
                CString::from_vec_unchecked(buffer)
            };
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            eprintln!("{}", error.to_string_lossy());
        }
        id
    }
}

fn create_program(
    vertex_shader: &str,
    fragment_shader: &str,
) -> u32 {
    let vs = create_shader(gl::VERTEX_SHADER, vertex_shader);
    let fs = create_shader(gl::FRAGMENT_SHADER, fragment_shader);
    
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        let mut success: gl::types::GLint = 1;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let error = {
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));
                CString::from_vec_unchecked(buffer)
            };
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            eprintln!("{}", error.to_string_lossy());
        }
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        program
    }

}

#[derive(Hash, PartialEq)]
struct FontCacheKey {
    c: String,
    color: Color,
}

struct FontCacheEntry {
    texture: SdlTexture,
    w: i32,
    h: i32,
}

impl Eq for FontCacheKey {}

pub struct App<'a> {
    // SDL
    sdl: Sdl,
    canvas: WindowCanvas,

    // OpenGL
    program: u32,
    program_2d: u32,
    program_text: u32,
    program_texture: u32,
    uniforms: Uniforms,
    rect_buffer: u32,
    rect_vertices: Vec<f32>,

    // Window
    pub window_width: u32,
    pub window_height: u32,

    // Text
    pub char_width: i32,
    pub font_size: i32,
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

    // Control flow
    pub should_quit: bool,
}

impl<'a> App<'a> {


    pub fn new(font_path: &str, font_size: u16) -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsys = sdl.video().unwrap();
        let window = video_subsys
            .window("SDL2_TTF Example", 800, 600)
            .position_centered()
            .resizable()
            .maximized()
            .opengl()
            .build()
            .unwrap();
        let canvas: WindowCanvas = window.into_canvas().build().unwrap();
        
        gl::load_with(|ptr| video_subsys.gl_get_proc_address(ptr) as *const _);
        canvas.window().gl_set_context_to_current().unwrap();

        unsafe {
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

        let char_width = font.glyph('o').scaled(Scale::uniform(font_size as f32)).h_metrics().advance_width as i32;

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

        let mut rect_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut rect_buffer);
        }

        Self {
            sdl,
            char_width,
            font_size: font_size as i32,
            window_width: 800,
            window_height: 600,
            font,
            font_cache: HashMap::new(),
            canvas,
            program,
            program_2d,
            program_text,
            program_texture,
            uniforms,
            mouse: Point::new(0, 0),
            scroll: Point::new(0, 0),
            mouse_left_down: false,
            mouse_left_pressed: false,
            mouse_right_down: false,
            mouse_right_pressed: false,
            mouse_middle_down: false,
            mouse_middle_pressed: false,
            should_quit: false,
            rect_buffer,
            rect_vertices: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.scroll.x = 0;
        self.scroll.y = 0;
        self.mouse_left_pressed = false;
        self.mouse_right_pressed = false;
        self.mouse_middle_pressed = false;
        for event in self.sdl.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => self.should_quit = true,
                Event::Window { win_event, .. } => {
                    match win_event {
                        WindowEvent::Resized(width, height) => self.resize(width as u32, height as u32),
                        _ => (),
                    }
                }
                Event::MouseWheel { x, y, .. } => {
                    self.scroll.x += x * 10;
                    self.scroll.y += y * 10;
                }
                Event::MouseMotion { x, y, .. } => {
                    self.mouse.x = x;
                    self.mouse.y = y;
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
                _ => (),
            }
        }
        // TODO
    }

    pub fn clear(&self, color: Color) {
        unsafe {
            let color = [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ];
            gl::ClearColor(color[0], color[1], color[2], color[3]);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn draw_rotated_rect(&self, rect: Rect, color: Color, origin: Point, rotation: f32) {

        let x = rect.x as f32;
        let y = rect.y as f32;
        let width = rect.width as f32;
        let height =  rect.height as f32;
        let dx = -origin.x as f32;
        let dy = -origin.y as f32;

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

        let (x1, x2, x3, x4, y1, y2, y3, y4) = (
            x1 * 2.0 / self.window_width as f32 - 1.0,
            x2 * 2.0 / self.window_width as f32 - 1.0,
            x3 * 2.0 / self.window_width as f32 - 1.0,
            x4 * 2.0 / self.window_width as f32 - 1.0,
            1.0 - y1 * 2.0 / self.window_height as f32,
            1.0 - y2 * 2.0 / self.window_height as f32,
            1.0 - y3 * 2.0 / self.window_height as f32,
            1.0 - y4 * 2.0 / self.window_height as f32,
        );

        let color = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];

        let vertices = [
            x1, y1, color[0], color[1], color[2], color[3],
            x2, y2, color[0], color[1], color[2], color[3],
            x4, y4, color[0], color[1], color[2], color[3],
            x1, y1, color[0], color[1], color[2], color[3],
            x4, y4, color[0], color[1], color[2], color[3],
            x3, y3, color[0], color[1], color[2], color[3],
        ];
        let (mut vao_2d, mut vbo_2d) = (0, 0);
        unsafe {
            gl::Disable(gl::DEPTH_TEST);

            gl::GenVertexArrays(1, &mut vao_2d);
            gl::GenBuffers(1, &mut vbo_2d);
            gl::BindBuffer(gl::ARRAY_BUFFER, vao_2d);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            gl::BindVertexArray(vao_2d);
            let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);

            gl::UseProgram(self.program_2d);
            gl::BindVertexArray(vao_2d);
            gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        unsafe {
            gl::DeleteVertexArrays(1, &mut vao_2d);
            gl::DeleteBuffers(1, &mut vbo_2d);
        }
    }

    pub fn draw_rect(&self, rect: Rect, color: Color) {
        self.draw_rotated_rect(rect, color, Point::new(0, 0), 0.0);
    }

    pub fn draw_rects(&mut self, rects: &[Rect], colors: &[Color], rotations: &[f32]) {

let mut start = Instant::now();

        let needs_allocation = rects.len() * 36 != self.rect_vertices.len();

        if needs_allocation {
            println!("Allocating rect vertices");
            self.rect_vertices = vec![0.0; rects.len() * 36];

        }
        // let mut vertices = vec![0.0; rects.len() * 36];

        for i in 0..rects.len() {
            let rect = rects[i];
            let color = colors[i];
            // TODO hack and wrong
            let origin = Point::new(rect.width as i32, rect.height as i32);
            let rotation = rotations[i];

            let x = rect.x as f32;
            let y = rect.y as f32;
            let width = rect.width as f32;
            let height =  rect.height as f32;
            // TODO hack and wrong
            let dx = -origin.x as f32 / 2.0;
            let dy = -origin.y as f32 / 2.0;

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

            let (x1, x2, x3, x4, y1, y2, y3, y4) = (
                x1 * 2.0 / self.window_width as f32 - 1.0,
                x2 * 2.0 / self.window_width as f32 - 1.0,
                x3 * 2.0 / self.window_width as f32 - 1.0,
                x4 * 2.0 / self.window_width as f32 - 1.0,
                1.0 - y1 * 2.0 / self.window_height as f32,
                1.0 - y2 * 2.0 / self.window_height as f32,
                1.0 - y3 * 2.0 / self.window_height as f32,
                1.0 - y4 * 2.0 / self.window_height as f32,
            );

            let color = [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ];

            let these_vertices = [
                x1, y1,
                x2, y2,
                x4, y4,
                x1, y1,
                x4, y4,
                x3, y3,
            ];
            
            for j in 0..these_vertices.len() {
                self.rect_vertices[i * 12 + j] = these_vertices[j];
            }
            if needs_allocation {
                for j in 0..6 {
                    self.rect_vertices[rects.len() * 12 + i * 24 + j * 4 + 0] = color[0];
                    self.rect_vertices[rects.len() * 12 + i * 24 + j * 4 + 1] = color[1];
                    self.rect_vertices[rects.len() * 12 + i * 24 + j * 4 + 2] = color[2];
                    self.rect_vertices[rects.len() * 12 + i * 24 + j * 4 + 3] = color[3];
                }
            }
        }

        let mut vao_2d = 0;
        unsafe {
            gl::Disable(gl::DEPTH_TEST);

            gl::GenVertexArrays(1, &mut vao_2d);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.rect_buffer);
            if !needs_allocation {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (rects.len() * 12 * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    self.rect_vertices.as_ptr() as *const _,
                );
            } else {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.rect_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    self.rect_vertices.as_ptr() as *const _,
                    gl::STATIC_DRAW
                );
            }

            gl::BindVertexArray(vao_2d);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, (rects.len() * 12 * mem::size_of::<GLfloat>()) as *const _);

            gl::UseProgram(self.program_2d);
            gl::BindVertexArray(vao_2d);

            gl::DrawArrays(gl::TRIANGLES, 0, self.rect_vertices.len() as GLsizei);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            gl::DeleteVertexArrays(1, &mut vao_2d);
        }

    }

    // TODO remove this function, figure out what we want to do with drawing 2d
    pub fn draw_2d(&self) {
        unsafe {
            // 2d
            gl::BindVertexArray(0);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
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

    pub fn start_3d(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
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
            gl::DrawArrays(gl::TRIANGLES, 0, vertex_buffer_length as GLsizei);
            // gl::BindVertexArray(0);
            gl::Disable(gl::DEPTH_TEST);
        }
    }

    pub fn draw_texture(&self, texture: &Texture, src_rect: Rect, dest_rect: Rect) {
        let x = dest_rect.x as f32 * 2.0 / self.window_width as f32 - 1.0;
        let y = 1.0 - dest_rect.y as f32 * 2.0 / self.window_height as f32;

        // Load the texture from the buffer
        let (uniform, mut id) = unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);

            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // TODO Decide what these should be.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                texture.width as GLint,
                texture.height as GLint,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                texture.data.as_ptr() as *const _
            );
            let uniform = gl::GetUniformLocation(self.program_texture, b"tex\0".as_ptr() as *const _);

            (uniform, id)
        };

        let dest_width = dest_rect.width as f32 * 2.0 / self.window_width as f32;
        let dest_height = dest_rect.height as f32 * 2.0 / self.window_height as f32;
        let y = y - dest_height;
        let u0 = src_rect.x as f32 / texture.width as f32;
        let u1 = (src_rect.x as f32 + src_rect.width as f32) / texture.width as f32;
        let v0 = src_rect.y as f32 / texture.height as f32;
        let v1 = (src_rect.y as f32 + src_rect.height as f32) / texture.height as f32;

        let vertices = [
            x, y, u0, v1,
            x + dest_width, y, u1, v1,
            x + dest_width, y + dest_height, u1, v0,
            x, y, u0, v1,
            x + dest_width, y + dest_height, u1, v0,
            x, y + dest_height, u0, v0,
        ];

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
            let stride = 4 * mem::size_of::<GLfloat>() as GLsizei;

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::Uniform1i(uniform, 0);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);

            gl::UseProgram(self.program_texture);

            gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        unsafe {
            gl::DeleteBuffers(1, &mut vbo);
            gl::DeleteVertexArrays(1, &mut vao);
            gl::DeleteTextures(1, &mut id);
        }
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

    pub fn draw_text(&self, text: &str, x: i32, y: i32, scale: f32, color: Color) -> Rect {
        // Save the original parameters to return in the rect
        let input_x = x;
        let input_y = y;

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
        let (uniform, mut id) = unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);

            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // TODO Decide what these should be.
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
            let uniform = gl::GetUniformLocation(self.program_text, b"tex\0".as_ptr() as *const _);
            (uniform, id)
        };

        let x = x as f32 * 2.0 / self.window_width as f32 - 1.0;
        let y = 1.0 - y as f32 * 2.0 / self.window_height as f32;
        let height = glyphs_height as f32 * 2.0 / self.window_height as f32;
        let width = glyphs_width as f32 * 2.0 / self.window_width as f32;
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
            let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::Uniform1i(uniform, 0);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (4 * mem::size_of::<GLfloat>()) as *const _);

            gl::UseProgram(self.program_text);

            gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        unsafe {
            gl::DeleteBuffers(1, &mut vbo);
            gl::DeleteVertexArrays(1, &mut vao);
            gl::DeleteTextures(1, &mut id);
            // gl::DeleteProgram(program);
        }

        Rect::new(input_x, input_y, glyphs_width as u32, glyphs_height as u32)
    }

    pub fn text_length(&self, text: &str) -> i32 {
        // TODO make this correct for non-monospace
        text.len() as i32 * self.char_width
        // let mut length = 0;
        // for c in text.chars() {
        //     let (x, _) = self.font.size_of_char(c).unwrap();
        //     length += x as i32;
        // }
        // length
    }

    pub fn set_font(&mut self, path: &Path, size: u16) {
        self.font = {
            let data = std::fs::read(path).unwrap();
            Font::try_from_vec(data).unwrap()
        };
        self.font_size = size as i32;
    }

    // pub fn set_active_region(&mut self, rect: Rect) {
    //     self.rect = rect;
    // }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}
