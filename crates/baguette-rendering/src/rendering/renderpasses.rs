pub use crate::*;

#[derive(Default)]
pub(crate) struct RenderPassCommands
{
    sprite_pass: Option<SpritePass>,
    tilemap_pass: Option<TilemapPass>,
}

impl RenderPassCommands
{
    pub fn add_sprite(&mut self, ctx: ContextHandle, sprite: SpriteBuilder) -> Sprite
    {
        let sprite_pass = self
            .sprite_pass
            .get_or_insert_with(Default::default);

        

        //for (&layer, ..) in sprite.sprite.layers.iter()
        //{
        //    match self.layers.get_mut(&layer)
        //    {
        //        Some((sprite, ..)) => *sprite = true,
        //        None =>
        //        {
        //            self.layers.insert(layer, (true, false));
        //        }
        //    } 
        //}

        sprite_pass.add_sprite(ctx, sprite)
    }

    pub fn add_tilemap(&mut self, ctx: &ContextHandleInner, tilemap: TilemapBuilder)
    {
        let tilemap_pass = self
            .tilemap_pass
            .get_or_insert_with(Default::default);

        #[allow(clippy::let_unit_value)]
        let tilemap = tilemap_pass.add(ctx, tilemap);

        //for (&layer, ..) in tilemap_pass.layers.iter()
        //{
        //    match self.layers.get_mut(&layer)
        //    {
        //        Some((.., tilemap)) => *tilemap = true,
        //        None =>
        //        {
        //            self.layers.insert(layer, (false, true));
        //        }
        //    } 
        //}

        tilemap
    }

    pub fn draw<'a>
    (
        &'a self, ctx: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData
    )
    {
        if let Some(sprite_pass) = &self.sprite_pass
        {
            sprite_pass.draw(ctx, pass, camera)
        }

        if let Some(tilemap_pass) = &self.tilemap_pass
        {
            tilemap_pass.draw(ctx, pass, camera)
        }
    }
    
    pub(crate) fn prepare(&mut self, _ctx: &ContextHandleInner)
    {
        if let Some(sprite_pass) = &mut self.sprite_pass
        {
            sprite_pass.prepare_instances();
        }     
    }

    pub fn resize(&mut self, ctx: &ContextHandleInner)
    {
        if let Some(pass) = &mut self.tilemap_pass
        {
            pass.resize(ctx)
        }
    }
}