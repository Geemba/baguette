pub use crate::*;

#[path = "2d/spritepass.rs"]
pub mod spritepass;
pub use spritepass::*;

#[path = "renderpasses/resolutionpass.rs"]
pub mod resolutionpass;

pub enum Passes
{
    SpriteSheet(SpritePass)
}

impl Passes
{
    pub(crate) fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError>
    {
        match self
        {
            Self::SpriteSheet(pass) => RenderPass::draw(pass, encoder, view),
        }
    }
}

pub(crate) struct RenderPasses
{
    pub renderpasses: Vec<Passes>
}

impl RenderPasses
{
    pub const fn new() -> Self { Self { renderpasses: vec![]} }
    
    /// immutable iteration
    pub fn iter(&self) -> std::slice::Iter<'_, Passes>
    {
        self.renderpasses.iter()
    }

    /// mutable iteration
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Passes>
    {
        self.renderpasses.iter_mut()
    }    
}

pub trait RenderPass
{
    fn add_pass() -> Passes where Self: Sized;
    
    #[allow(clippy::cast_possible_truncation)]
    fn draw
    (
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView
        
    ) -> Result<(), wgpu::SurfaceError>;
}