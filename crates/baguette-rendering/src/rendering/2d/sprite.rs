use std::{ptr::NonNull, sync::RwLockReadGuard};

use crate::*;

#[must_use]
/// runtime instance of a sprite, contains both the texture and all the instances
pub struct Sprite
{
    pub(crate) sprite: Box<SpriteImpl>,
    ///// this is used only on drop to remove the reference to this [Sprite]
    pub(crate) sprites: NonNull<Vec<NonNull<SpriteImpl>>>
}

impl Sprite
{
    pub fn new (renderer: &mut crate::Renderer, loader: crate::SpriteLoader) -> Self
    {
        renderer.load_sprite(loader)
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
                //.expect
                //(
                //    "attempted to remove a sprite,
                //    but the id dind't correspond to anything"
                //);
        }          
    }
}

impl std::ops::Index<usize> for Sprite
{
    type Output = SpriteInstance;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.instances[index]
    }
}

// index mut very likely sucks performance whise
// since it both iterates and recreates the instance buffer for just one item mutation
impl std::ops::IndexMut<usize> for Sprite
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        self.iter_mut().nth(index).unwrap()
    }
}

pub struct SpriteImpl
{
    pub(crate) instances: Vec<SpriteInstance>,
    pub(crate) slice: SpriteSlice,
    pub(crate) pivot: Option<Vec2>,

    /// the texture that the sprite will use
    pub(crate) texture: TextureData,
}

/// impl containing sprite loading
impl SpriteImpl
{
    /// loads a [`SpriteSheetBinding`] from a [crate::SpriteLoader].
    ///
    /// panics if the path is not found
    pub fn from_loader(ctx: &RwLockReadGuard<ContextHandleData>, loader: SpriteLoader) -> Self
    {
        let SpriteLoader { ref path, filtermode, pivot, mut instances, pxunit, rows, columns } = loader;

        let image = image::io::Reader::open(path)
            .unwrap()
            .decode()
            .expect("failed to decode image, unsupported format");

        // if we need to rescale we need to do it on the dyn image and not this variable
        // otherwhise we just crop the rendered texture
        let dimensions = Into::<baguette_math::UVec2>::into
        (
            image::GenericImageView::dimensions(&image)
        );

        let size = wgpu::Extent3d
        {
            width: dimensions.x,
            height: dimensions.y,
            depth_or_array_layers: 1
        };

        let texture = ctx.create_texture
        (
            wgpu::TextureDescriptor
            {
                size,
                // the label is the directory of the sprite we loaded
                label: Some(&path.to_string_lossy()),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[]
            }
        );

        ctx.write_texture
        (
            wgpu::ImageCopyTexture 
            {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO
            },
            &image.to_rgba8(),
            wgpu::ImageDataLayout
            {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.x),
                rows_per_image: Some(dimensions.y)
            },
            size
        );

        let view = texture.create_view(&Default::default());
        let sampler = ctx.create_sampler
        (
            wgpu::SamplerDescriptor
            {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: filtermode,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        // we adjust the dimensions of the vertex positions using the 
        // pixel per unit factor
        let scale = Vec2::new
        (
            dimensions.x as f32 / pxunit,
            dimensions.y as f32 / pxunit
        );

        let vertices =
        [
            [-scale.x, scale.y],
            [-scale.x, -scale.y],
            [scale.x, -scale.y],
            [scale.x, scale.y]
        ];

        let texture = crate::TextureData { texture, view, sampler, pxunit };

        let slice = SpriteSlice::new(vertices, rows, columns);

        Self
        {
            instances,
            texture,
            pivot,
            slice,
        }
    }
}

impl SpriteImpl
{
    /// iters the instances mutably
    pub fn iter_mut(&mut self) -> SpriteIterMut
    {
        SpriteIterMut
        {
            instances: (&mut self.instances).into(),
            iter_index: 0,
            phantom: std::marker::PhantomData,
        }
    }

    /// iters the instances immutably
    pub fn iter(&self) -> std::slice::Iter<'_, SpriteInstance>
    {
        self.instances.iter()
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
}

impl Default for SpriteInstance
{
    fn default() -> Self
    {
        Self
        {
            translation: Vec3::default(),
            orientation: Quat::default(),
            scale: Vec3::ONE
        }
    }
}

impl SpriteInstance
{
    #[inline]
    pub(crate) fn as_raw(&self, pivot: Option<Vec2>, bind_idx: u32) -> SpriteInstanceRaw
    {
        SpriteInstanceRaw
        {
            transform:
            {
                match pivot
                {
                    Some(pivot) =>
                    {
                        Mat4::from_translation(vec3(pivot.x, pivot.y, 0.)) *
                        Mat4::from_scale(self.scale) * 
                        Mat4::from_quat(self.orientation) * 
                        Mat4::from_translation(-vec3(pivot.x, pivot.y, 0.)) *
                        Mat4::from_translation(self.translation)
                    }
                    None => Mat4::from_scale_rotation_translation
                    (
                        self.scale, self.orientation, self.translation
                    )
                }

            }.to_cols_array_2d(),
            uv_idx: 0,
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

pub struct SpriteIterMut<'a>
{
    instances: NonNull<Vec<SpriteInstance>>,

    iter_index: usize,
    phantom: std::marker::PhantomData<&'a ()>
}

impl<'a> Iterator for SpriteIterMut<'a>
{
    type Item = &'a mut SpriteInstance;

    fn next(&mut self) -> Option<Self::Item>
    {
        // SAFETY: we know the lifetime will live longer than this function
        // since the vec is stored in the renderer
        let next = unsafe 
        {
            self.instances
                .as_mut()
                .get_mut(self.iter_index)
        };
        
        self.iter_index += 1;

        next
    }
}

pub(crate) const SPRITE_INDICES_U32: [u32; 6] =
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
