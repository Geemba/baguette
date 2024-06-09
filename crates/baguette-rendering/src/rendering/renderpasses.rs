pub use crate::*;


#[derive(Default)]
pub(crate) struct RenderPassCommands
{
    sprite_pass: Option<SpritePass>,
    tilemap_pass: Option<TilemapPass>,
    layers: FastIndexMap<u8, (bool, bool)>
}

impl RenderPassCommands
{
    type Target = Vec<Pass>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl std::ops::DerefMut for RenderPasses
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub(crate) enum Pass
{
    Sprite(SpritePass),
    Tilemap(TilemapPass)
}
impl Pass 
{
    pub(crate) fn draw<'a>
    (
        &'a mut self, ctx: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData
    ) -> Result<(), wgpu::SurfaceError>
    {
        match self
        {
            Pass::Sprite(sprite_pass) => sprite_pass.draw(ctx, pass, camera),
            Pass::Tilemap(tilemap_pass) => tilemap_pass.draw(ctx, pass, camera),
        }
    }
}

impl From<SpritePass> for Pass
{
    fn from(pass: SpritePass) -> Self
    {
        Self::Sprite(pass)
    }
}

impl From<TilemapPass> for Pass
{
    fn from(pass: TilemapPass) -> Self
    {
        Self::Tilemap(pass)
    }
}

/// a trait to implement draw commands 
pub(crate) trait DrawPass: Into<Pass> + Default
{    
    #[allow(clippy::cast_possible_truncation)]
    fn draw<'a>
    (
        &'a mut self,
        ctx: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a camera::CameraData
    ) -> Result<(), wgpu::SurfaceError>;
}