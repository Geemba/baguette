pub use crate::*;

use std::sync::Arc;
use parking_lot::*;

#[derive(Default)]
pub(crate) struct RenderPassCommands
{
    sprite_pass: Option<SpritePass>,
    tilemap_pass: Option<TilemapPass>,
    layers: Layers2D,
}

impl RenderPassCommands
{
    pub fn add_sprite(&mut self, ctx: ContextHandle, sprite: SpriteBuilder) -> Sprite
    {
        let sprite_pass = self.sprite_pass.get_or_insert_with(Default::default);

        let sprite = sprite_pass.add_sprite(ctx, sprite, self.layers.clone());
        let mut layers = self.layers.write();

        for (&layer, instances) in sprite.iter_all()
        {
            let sprite_instances_to_add = instances.len();

            if let Some(draw_op) = layers.get_mut(&layer)
            {
                *draw_op = match draw_op
                {
                    Pass::Sprite(count) => Pass::Sprite(*count + sprite_instances_to_add),
                    Pass::TileMap(count) => 
                    {
                        Pass::SpriteAndTilemap
                        {
                            sprite: sprite_instances_to_add,
                            tilemap: *count
                        }
                    },
                    Pass::SpriteAndTilemap { sprite, tilemap } =>
                    {
                        Pass::SpriteAndTilemap
                        {
                            sprite: *sprite + sprite_instances_to_add,
                            tilemap: *tilemap
                        }
                    },
                }
            }
            else
            {
                layers.insert(layer, Pass::Sprite(sprite_instances_to_add));
            } 
        }

        sprite
    }

    pub fn add_tilemap(&mut self, ctx: &ContextHandleInner, tilemap: TilemapBuilder)
    {
        let tilemap_pass = self.tilemap_pass.get_or_insert_with(Default::default);

        #[allow(clippy::let_unit_value)]
        let tilemap = tilemap_pass.add(ctx, tilemap);
        let mut layers = self.layers.write();

        // todo use tilemap handle metod to iter
        for (layer, instances) in tilemap_pass.layers.iter()
        {
            let tilemap_instances_to_add = instances.len();

            if let Some(drawop) = layers.get_mut(layer)
            {
                *drawop = match drawop
                {
                    Pass::Sprite(count) => Pass::SpriteAndTilemap { sprite: *count, tilemap: tilemap_instances_to_add },
                    Pass::TileMap(count) => Pass::TileMap(*count + tilemap_instances_to_add),
                    Pass::SpriteAndTilemap { sprite, tilemap } => Pass::SpriteAndTilemap
                    {
                        sprite: *sprite, tilemap: *tilemap + tilemap_instances_to_add
                    },
                };
            }
            else
            {
                layers.insert
                (
                    *layer, Pass::TileMap(tilemap_instances_to_add)
                );
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
        for (&layer, draw_op) in self.layers.read().iter()
        {
            match draw_op
            {
                Pass::Sprite(..) =>
                {
                    self.sprite_pass.as_ref().expect
                    (
                        "draw op was set to sprite,
                        meaning the sprite pass should have been present"
                    )
                    .draw(ctx, pass, camera, layer);
                }

                Pass::TileMap(..) =>
                {
                    self.tilemap_pass.as_ref().expect
                    (
                        "draw op was set to tilemap,
                        meaning the tilemap pass should have been present"
                    )
                    .draw(ctx, pass, camera, layer)
                }

                Pass::SpriteAndTilemap { .. } =>
                {
                    self.sprite_pass.as_ref().expect
                    (
                        "draw op was set to both sprite and tilemap,
                        meaning the sprite pass should have been present"
                    )
                    .draw(ctx, pass, camera, layer);

                    self.tilemap_pass.as_ref().expect
                    (
                        "draw op was set to both sprite and tilemap,
                        meaning the tilemap pass should have been present"
                    )
                    .draw(ctx, pass, camera, layer)
                }
            }
        }
    }
    
    pub(crate) fn prepare(&mut self, _ctx: &ContextHandleInner)
    {
        if let Some(sprite_pass) = &mut self.sprite_pass
        {
            sprite_pass.prepare_instances();
        }     
    }

    pub fn resize(&mut self, _ctx: &ContextHandleInner)
    {
        //self.binding = Binding::new(ctx)
    }
}

/// describes which layers are present and have to be drawn
#[derive(Default, Clone)]
pub struct Layers2D(Arc<RwLock<FastIndexMap<u8, Pass>>>);

impl Layers2D
{
    pub fn read(&self) -> RwLockReadGuard<FastIndexMap<u8, Pass>>
    {
        self.0.read()
    }

    pub fn write(&self) -> RwLockWriteGuard<FastIndexMap<u8, Pass>>
    {
        self.0.write()
    }
}

/// describes which pass has to be drawn,
/// 
/// draws the pass until the count is zero
#[derive(Debug)]
pub enum Pass
{
    Sprite(usize),
    TileMap(usize),
    /// need to draw both tilemaps and sprites
    SpriteAndTilemap
    {
        /// the amount of sprite instances
        sprite: usize,
        /// the amount of tile instances
        tilemap: usize
    }
}

//pub enum DrawResult
//{
//    /// drawed
//    Ok,
//    // no pass was in the layer
//    DidNotDraw,
//    Error
//}
