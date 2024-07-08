use std::ptr::NonNull;
use crate::*;

type Owner = NonNull<Vec<NonNull<SpriteImpl>>>;

#[must_use]
/// runtime instance of a sprite, contains both the texture and all the instances
pub struct Sprite
{
    pub(crate) sprite: Box<SpriteImpl>,
    ///// this is used only on drop to remove the reference to this [Sprite]
    pub(crate) sprites: Owner
}

impl Sprite
{
    pub fn new (renderer: &mut crate::Renderer, builder: crate::SpriteBuilder) -> Self
    {
        renderer.add_sprite(builder)
    }
}

impl std::ops::Deref for Sprite
{
    type Target = SpriteImpl;

    fn deref(&self) -> &Self::Target
    {
        self.sprite.as_ref()
    }
}

impl std::ops::DerefMut for Sprite
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        self.sprite.as_mut()
    }
}

impl Drop for Sprite
{
    fn drop(&mut self)
    {
        // remove the reference to this sprite
        // from the spritepass as it's about to be dropped.
        unsafe 
        {
            self.sprites.as_mut()
            .retain
            (
                |sprite| (&mut *self.sprite) as *mut SpriteImpl == sprite.as_ptr()
            )
        }
    }
}

impl Sprite
{
    /// iters the instances immutably
    pub fn iter_all(&self) -> IterAllLayers
    {
        // SAFETY: data access is locked before being accessed
        // and will be unlocked by the destructor
        unsafe
        {
            let lock = self.handle.raw();

            lock.lock_shared();

            let sprite = &self.handle.as_ref().sprites[&self.id];

            let iter = sprite.layers.iter();

            IterAllLayers
            {
                iter,
                lock,
            }
        }
    }

    /// iters every layers instances mutably
    pub fn iter_all_mut(&mut self) -> IterAllLayersMut
    {
        // SAFETY: data access is locked before being accessed
        // and will be unlocked by the destructor
        unsafe
        {
            let lock = self.handle.raw();

            lock.lock_exclusive();
      
            let sprite = &mut self.handle.as_ptr().as_mut().sprites[&self.id];

            let iter = sprite.layers.iter_mut();
            
            let binding = &mut self.handle;

            IterAllLayersMut
            {
                iter,
                handle: binding,
                ctx: self.ctx.clone(),
            }               
        }
    }
    
    /// iters the instances of a layer immutably
    pub fn iter_layer(&self, layer: u8) -> IterLayer
    {
        // SAFETY: data access is locked before being accessed
        // and will be unlocked by the destructor
        unsafe
        {
            let lock = self.handle.raw();

            lock.lock_shared();
    
            let sprite = &self.handle.as_ref().sprites[&self.id];

            let iter = sprite.layers[&layer].iter();

            IterLayer
            {
                iter,
                lock,
            }
        }
    }

    /// iters the instances of a layer mutably
    pub fn iter_layer_mut(&mut self, layer: u8) -> IterLayerMut
    {
        // SAFETY: data access is locked before being accessed
        // and will be unlocked by the destructor
        unsafe
        {
            let lock = self.handle.raw();

            lock.lock_exclusive();
            
            let sprite = &mut self.handle.as_ptr().as_mut().sprites[&self.id];

            let iter = sprite.layers[&layer].iter_mut();

            IterLayerMut
            {
                iter,
                lock,
            }
        }
    }

    /// returns the size of the texture
    pub fn size(&self) -> baguette_math::Vec2
    {
        self.texture.size()
    }

    //pub fn add_instances(&mut self, ctx: &crate::ContextHandleData, mut new_instances: Vec<SpriteInstance>)
    //{
    //    // we add the new instances
    //    self.instances.append(&mut new_instances);         

    //    // .. then recreate the buffer to update it with the added instance
    //    let data: Vec<SpriteInstanceRaw> = self.instances.iter()
    //        .map(|f| f.as_raw())
    //        .collect();

    //    ctx.write_buffer(&self.binding.instance_buffer.0, &data)
    //}
}

#[derive(Clone, Debug)]
/// represents a single instance of a sprite
pub struct SpriteInstance
{
    pub translation: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
    /// indicates the index to which tile is being rendered if the sprite is sliced,
    /// if not, it won't do anything
    pub uv_idx: u32,
}

impl Default for SpriteInstance
{
    fn default() -> Self
    {
        Self
        {
            translation: Vec3::default(),
            orientation: Quat::default(),
            scale: Vec3::ONE,
            uv_idx: u32::default(),
        }
    }
}

impl SpriteInstance
{
    #[inline]
    pub(crate) fn as_raw(&self, slice: &SpriteSlice, pivot: Option<Vec2>, bind_idx: u32) -> SpriteInstanceRaw
    {
        SpriteInstanceRaw
        {
            transform:
            {
                match pivot
                {
                    Some(pivot) =>
                    {
                        Mat4::from_scale_rotation_translation
                        (
                            self.scale, self.orientation, vec3(pivot.x, pivot.y, 0.)
                        ) *
                        Mat4::from_translation(self.translation + -vec3(pivot.x, pivot.y, 0.))
                    }
                    None => Mat4::from_scale_rotation_translation
                    (
                        self.scale, self.orientation, self.translation
                    )
                }

            }.to_cols_array_2d(),
            uv_idx: u32::min(self.uv_idx, slice.rows * slice.columns - 1),
            bind_idx,
        }
    }

    #[inline]
    /// rotates along the y axis to face the camera
    pub fn billboard_y(&mut self, cam: &mut crate::Camera)
    {
        self.orientation = cam.data.borrow().orientation();
        self.orientation.y *= -1.;
    }

    #[inline]
    /// rotates along the x and y axis to face the camera
    pub fn billboard_xy(&mut self, cam: &mut crate::Camera)
    {
        self.orientation = cam.data.borrow().orientation().inverse()
    }
}

/// the number of rows and columns that subdivide this spritesheet 
#[derive(Clone, Copy)]
pub struct SpriteLayout
{
    /// how many rows are in the spritesheet
    pub rows: u32,
    /// how many columns are in the spritesheet
    pub columns: u32,
}

impl Default for SpriteLayout
{
    fn default() -> Self 
    {
        Self { rows: 1, columns: 1 }
    }
}

pub(crate) const SPRITE_INDICES_U32: [u32; 6] =
[
    0, 1, 2, 2, 3, 0
];

pub(crate) const SPRITE_INDICES_U16: [u16; 6] =
[
    0, 1, 2, 2, 3, 0
];

/// describes the sorting order, if present, of the sprites
pub enum SpriteSorting
{
    X,
    Y,
    Z,
    None
}

/// iters all the layers of the sprite immutably.
pub struct IterAllLayers<'a>
{
    iter: indexmap::map::Iter<'a, u8, Vec<SpriteInstance>>,
    lock: &'a parking_lot::RawRwLock
}

impl<'a> Iterator for IterAllLayers<'a>
{
    type Item = (&'a u8, &'a Vec<SpriteInstance>);

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}

impl<'a> Drop for IterAllLayers<'a>
{
    fn drop(&mut self)
    {
        // Safety: we lock the access when creating 
        // an iterator so we must unlock it when dropping
        unsafe { self.lock.unlock_shared() }
    }
}

/// iters all the layers of the sprite.
/// 
/// safely unlocks the shared lock when dropped
pub struct IterAllLayersMut<'a>
{
    iter: indexmap::map::IterMut<'a, u8, Vec<SpriteInstance>>,
    ctx: ContextHandle,
    handle: &'a mut Handle,
}

impl<'a> Iterator for IterAllLayersMut<'a>
{
    type Item = (&'a u8, &'a mut Vec<SpriteInstance>);

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}

impl<'a> Drop for IterAllLayersMut<'a>
{
    fn drop(&mut self)
    {
        self.handle.update_binding(&self.ctx.read());

        // Safety: we lock the access when creating 
        // an iterator so we must unlock it when dropping
        unsafe { self.handle.raw().unlock_exclusive() }
    }
}

pub struct IterLayer<'a>
{
    iter: std::slice::Iter<'a, SpriteInstance>,
    lock: &'a parking_lot::RawRwLock
}

impl<'a> Iterator for IterLayer<'a>
{
    type Item = &'a SpriteInstance;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}

impl<'a> Drop for IterLayer<'a>
{
    fn drop(&mut self)
    {
        // Safety: we lock the access when creating 
        // an iterator so we must unlock it when dropping
        unsafe { self.lock.unlock_shared() }
    }
}

pub struct IterLayerMut<'a>
{
    iter: std::slice::IterMut<'a, SpriteInstance>,
    lock: &'a parking_lot::RawRwLock
}

impl<'a> Iterator for IterLayerMut<'a>
{
    type Item = &'a mut SpriteInstance;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}

impl<'a> Drop for IterLayerMut<'a>
{
    fn drop(&mut self)
    {
        // Safety: we lock the access when creating 
        // an iterator so we must unlock it when dropping
        unsafe { self.lock.unlock_exclusive() }
    }
}