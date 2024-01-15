// contains an integration of egui specifically for the baguette engine
// this is mostly taken by the official wgpu integration

#[allow(dead_code)]
mod egui_wgpu;
#[allow(dead_code)]
mod egui_winit;

pub use egui;

pub struct Ui<'a>
{
    handle: &'a UiHandle
}

impl Ui<'_>
{
    pub fn context(&self) -> &egui::Context
    {
        &self.handle.state.ctx
    }
}

impl<'a> From<&'a UiHandle> for Ui<'a>
{
    fn from(handle: &'a UiHandle) -> Self
    {
        Self { handle }
    }
}

/// Ui renderer
pub struct UiHandle
{
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    screen: ScreenData,
    tex_to_remove: Vec<egui::TextureId>
}

#[derive(Clone, Copy)]
struct ScreenData
{
    width: u32,
    height: u32,
    /// scale factor
    scale: f32
}

impl UiHandle
{
    pub fn new(width: u32, height: u32, scale: f32) -> Self
    {
        Self
        {
            state: egui_winit::State::new(Some(scale), None),
            renderer: egui_wgpu::Renderer::new
            (
                crate::device(), wgpu::TextureFormat::Bgra8UnormSrgb, None, 1
            ),
            screen: ScreenData
            {
                width,
                height,
                scale
            },
            tex_to_remove: Vec::new(),
        }
    }

    pub(crate) fn render<'a>
    (
        &'a mut self, pass: &mut wgpu::RenderPass<'a>,
        window: &crate::Window
    )
    {
        let id = &self.state.ctx.viewport_id();
        let mut output = self.state.ctx.end_frame();

        let commands = &output.viewport_output
            .get_mut(id)
            .expect("the context's viewport id didn't match any actual viewport")
            .commands;

        self.state.process_viewport_commands(commands, window);

        self.state.handle_platform_output(window, output.platform_output);

        let clipped_primitives = &self.state.ctx.tessellate
        (
            output.shapes, self.screen.scale
        );
        for (id, ref delta) in output.textures_delta.set
        {
            self.renderer.update_texture(crate::device(), crate::queue(), id, delta)
        }

        self.renderer.update_buffers
        (
            crate::device(), crate::queue(),
            &mut crate::create_command_encoder("update egui buffers"),
            clipped_primitives, self.screen
        );

        self.renderer.render(pass, clipped_primitives, &self.screen);
    }

    pub fn begin_egui_frame(&mut self, window: &input::winit::window::Window)
    {
        // prepare the gathered input
        self.state.update_viewport_info(window);
        let input = self.state.take_egui_input(window);

        self.state.ctx.begin_frame(input)
    }

    /// checks on input on the ui and passes it
    /// to the rest of the program if it's not consumed 
    pub fn handle_input(&mut self, window: &crate::Window, event: &input::WindowEvent) -> egui_winit::EventResponse
    {
        self.state.on_window_event(window, event)
    }

    pub(crate) fn update_screen_size(&mut self, width: u32, height: u32)
    {
        self.screen.width = width;
        self.screen.height = height;
    }
    
    pub fn free_textures(&mut self)
    {
        for id in self.tex_to_remove.drain(..)
        {
            self.renderer.free_texture(&id)
        }
    }
}