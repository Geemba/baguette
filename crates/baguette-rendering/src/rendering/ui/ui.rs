// contains an integration of egui specifically for the baguette engine
// this is mostly taken by the official wgpu integration

#[allow(dead_code)]
mod egui_wgpu;
pub use egui;

/// Ui renderer
pub struct Ui
{
    ctx: egui::Context,
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

impl Ui
{
    pub fn new(width: u32, height: u32, scale: f32) -> Self
    {
        Self
        {
            ctx: Default::default(),
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
        &'a mut self, pass: &mut wgpu::RenderPass<'a>
    )
    {
        egui::Window::new("title")
            .default_height(100.)
            .frame(egui::Frame
            {
                fill: egui::Color32::from_rgb(10, 10, 10),
                inner_margin: egui::Margin::same(3.),
                rounding: egui::Rounding::ZERO.at_least(5.),
                ..Default::default()
            })
            .show(&self.ctx, |ui| 
            {
                ui.label(egui::RichText::new("Large text").font(egui::FontId::proportional(40.0)));
            });

        let output = self.ctx.end_frame();

        let clipped_primitives = &self.ctx.tessellate
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

    pub fn free_textures(&mut self)
    {
        for id in &self.tex_to_remove
        {
            self.renderer.free_texture(id)
        }
    }

    pub fn begin_frame(&self, input: egui::RawInput)
    {
        self.ctx.begin_frame(input)
    }

    pub(crate) fn update_screen_size(&mut self, width: u32, height: u32)
    {
        self.screen.width = width;
        self.screen.height = height;
    }
}