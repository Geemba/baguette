pub use crate::*;

pub enum Passes
{
    SpriteSheet(SpritePass),
}

impl Passes
{
    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) -> Result<(), wgpu::SurfaceError>
    {
        match self
        {
            Self::SpriteSheet(pass) => pass as &dyn RenderPass,
        }.draw(pass)
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

    ///// mutable iteration
    //pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Passes>
    //{
    //    self.renderpasses.iter_mut()
    //}    
}

pub trait RenderPass
{
    /// describes how to initialize this pass
    fn add_pass() -> Passes where Self: Sized;
    
    #[allow(clippy::cast_possible_truncation)]
    fn draw<'a>
    (
        &'a self,
        pass: &mut wgpu::RenderPass<'a>        
    ) -> Result<(), wgpu::SurfaceError>;
}