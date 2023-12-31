// contains an integration of egui specifically for the baguette engine
// this is mostly taken by the official wgpu integration

mod egui_wgpu;
pub use egui;

/// Ui renderer
pub struct Ui
{
    ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    screen: ScreenData,

    texture: (wgpu::Texture, wgpu::TextureView),
}

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
        let texture = crate::create_texture(wgpu::TextureDescriptor
        {
            label: Some("ui view"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

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
            texture: (texture, view),
        }
    }

    pub(crate) fn render<'a>
    (
        &'a self, pass: &mut wgpu::RenderPass<'a>, surf_tex: &wgpu::Texture, view: &wgpu::TextureView
    )
    {
        let output = self.ctx.end_frame();
        
        let clipped_primitives = &self.ctx.tessellate
        (
            output.shapes, self.screen.scale
        );

        self.renderer.render(pass, clipped_primitives, &self.screen);
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