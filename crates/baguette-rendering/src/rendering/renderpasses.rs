use wgpu::*;

pub use crate::*;

#[path = "renderpasses/spritepass.rs"]
pub mod spritepass;
#[path = "renderpasses/resolutionpass.rs"]
pub mod resolutionpass;

pub struct RenderPasses
{
    renderpasses : Vec<Box<dyn RenderPass>>
}

impl RenderPasses 
{
    pub fn new() -> Self { Self { renderpasses : vec![]} }

    pub fn add_pass<'a>(&mut self, pass : impl RenderPass + 'a + 'static)
    {
        self.renderpasses.push(Box::new(pass))
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Box<dyn RenderPass>>
    {
        self.renderpasses.iter_mut()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Box<dyn RenderPass>>
    {
        self.renderpasses.iter()
    }
}

pub trait RenderPass
{
    fn draw
    (
        &self,
        encoder : &mut CommandEncoder,
        view : &TextureView
        
    ) -> Result<(), wgpu::SurfaceError>;
}