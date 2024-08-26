use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::video::Window;

use super::imgui_sdl2_support;

pub struct Imgui {
    imgui: imgui::Context,
    imgui_sdl2: imgui_sdl2_support::SdlPlatform,
    imgui_renderer: imgui_glow_renderer::AutoRenderer,
}

impl Imgui {
    pub fn new(window: &Window) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let imgui_sdl2 = imgui_sdl2_support::SdlPlatform::init(&mut imgui);
        let glow_context = unsafe {
            imgui_glow_renderer::glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
        };
        let imgui_renderer = imgui_glow_renderer::AutoRenderer::initialize(glow_context, &mut imgui).unwrap();

        Self {
            imgui,
            imgui_sdl2,
            imgui_renderer,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.imgui_sdl2.handle_event(&mut self.imgui, &event);
    }


    pub fn prepare_frame(&mut self, window: &Window, event_pump: &EventPump) {
        self.imgui_sdl2.prepare_frame(&mut self.imgui, &window, &event_pump);
    }

    pub fn render(&mut self) {
        self.imgui_renderer.render(self.imgui.render()).unwrap();
    }

    pub fn new_frame(&mut self) -> &mut imgui::Ui {
        self.imgui.new_frame()
    }
}

