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
    pub fn add_sprite(&mut self, ctx: &ContextHandleInner, sprite: SpriteBuilder) -> Sprite
    {
        let sprite_pass = self
            .sprite_pass
            .get_or_insert_with(Default::default);

        let sprite = sprite_pass.add_sprite(ctx, sprite);

        for (&layer, ..) in sprite.sprite.layers.iter()
        {
            match self.layers.get_mut(&layer)
            {
                Some((sprite, ..)) => *sprite = true,
                None =>
                {
                    self.layers.insert(layer, (true, false));
                }
            } 
        }

        sprite
    }

    pub fn add_tilemap(&mut self, ctx: &ContextHandleInner, tilemap: TilemapBuilder<FullyConstructed>)
    {
        let tilemap_pass = self
            .tilemap_pass
            .get_or_insert_with(Default::default);

        #[allow(clippy::let_unit_value)]
        let tilemap = tilemap_pass.add(ctx, tilemap);

        for (&layer, ..) in tilemap_pass.layers.iter()
        {
            match self.layers.get_mut(&layer)
            {
                Some((.., tilemap)) => *tilemap = true,
                None =>
                {
                    self.layers.insert(layer, (false, true));
                }
            } 
        }

        tilemap
    }

    pub fn draw<'a>
    (
        &'a self, ctx: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData
    )
    {
        // draw the tilemap behind and draw the sprites on top for each layer
        for (&layer, &(sprite_layer, tilemap_layer)) in self.layers.iter()
        {
            if let Some(sprite_pass) = &self.sprite_pass
            {
                if sprite_layer
                {
                    sprite_pass.draw(ctx, pass, camera, layer);
                }
            }

            if let Some(tilemap_pass) = &self.tilemap_pass
            {
                if tilemap_layer
                {
                    tilemap_pass.draw(ctx, pass, camera, layer);
                }
            }
        };
    }
    
    pub(crate) fn prepare(&mut self, _ctx: &ContextHandleInner)
    {
        if let Some(sprite_pass) = &mut self.sprite_pass
        {
            sprite_pass.prepare_instances();
        }     
    }
}