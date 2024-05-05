pub use crate::*;

pub enum Passes
{
    /// pass tasked with rendering sprites
    SpriteSheet(SpritePass),
}

impl Passes
{
    pub(crate) fn draw<'a>
    (
        &'a self, pass: &mut wgpu::RenderPass<'a>, camera: &'a camera::CameraData
    )
    -> Result<(), wgpu::SurfaceError>
    {
        match self
        {
            Self::SpriteSheet(pass) => pass as &dyn RenderPass,
        }.draw(pass, camera)
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

pub(crate) trait RenderPass
{
    /// describes how to initialize this pass
    fn add_pass(ctx: ContextHandle) -> Passes where Self: Sized;
    
    #[allow(clippy::cast_possible_truncation)]
    fn draw<'a>
    (
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a camera::CameraData
    ) -> Result<(), wgpu::SurfaceError>;
}