pub struct SpriteBinding
{
    pub(crate) instances: Vec<SpriteInstance>,

    /// the texture that the sprite will use
    pub(crate) texture: crate::Texture,

    pub(crate) binding: Box<SpriteGpuBinding>,
}

/// represents a single instance of a sprite
pub struct SpriteInstance
{
    /// the transform matrix of this instance
    pub transform: crate::Transform,
    /// the section to render if this is a spritesheet
    pub section: SheetSection
}

impl SpriteInstance
{
    #[inline]
    fn as_raw(&self) -> SpriteInstanceRaw
    {
        SpriteInstanceRaw
        {
            transform: self.transform.as_raw(),
            index: self.section.index
        }
    }

    #[inline]
    /// rotates along the y axis to face the camera
    pub fn billboard_y(&mut self, cam: &mut crate::Camera)
    {
        self.transform.orientation = cam.orientation();
        self.transform.orientation.y *= -1.;
    }

    #[inline]
    /// rotates along the x and y axis to face the camera
    pub fn billboard_xy(&mut self, cam: &mut crate::Camera)
    {
        self.transform.orientation = cam.orientation().inverse()
    }

    pub fn position(&self) -> baguette_math::Vec3
    {
        self.transform.translation
    }

    pub fn scale(&self) -> baguette_math::Vec3
    {
        self.transform.scale
    }

    pub fn orientation(&self) -> baguette_math::math::Quat
    {
        self.transform.orientation
    }
}

#[derive(Clone)]
pub struct Tiles { index: usize, indices: Box<[u32]> }

//impl Tiles
//{
//    pub fn new(indices: impl IntoIndices) -> Option<Self>
//    {
//        indices.into_indices(layout)
//    }
//}

#[derive(Clone)]
pub struct SheetSection
{
    /// describes the layout of the sheet for bound checking
    /// and for index multiplication, since we need to know the amount of 
    /// sections per row we keep this value around
    layout: SpriteLayout,
    indices: Option<Tiles>,
    /// the index into the uv buffer for this section
    index: u32
}

impl SheetSection
{
    pub fn empty() -> Self
    {
        Self
        {
            layout: SpriteLayout{ rows: 1, columns: 1 },
            indices: None,
            index: 0,
        }
    }

    /// sets the index of the sheet to render with the provided row and column value
    pub fn set(&mut self, row: u32, column: u32)
    {
        // we clamp the max value possible to the length of the uv buffer, whose value is
        // determined by (rows * columns -1 ) * 4
        self.index = u32::min
        (
            column * self.layout.rows + row,
            (self.layout.rows * self.layout.columns) -1
        ) * 4
    }

    /// sets the index of the spritesheet's section to the next one
    /// or the first one if it exceeds the maximum avaiable index
    pub fn next_or_first(&mut self)
    {
        self.index = match self.indices
        {
            Some(ref mut tiles) => match tiles.indices.get(tiles.index + 1)
            {
                Some(&next_index) =>
                {
                    tiles.index += 1;
                    next_index * 4
                }
                None =>
                {
                    tiles.index = 0;
                    tiles.indices[0] * 4
                }
            }
            
            None => match self.index + 4 <= (self.layout.rows * self.layout.columns - 1) * 4
            {
                true => self.index + 4,
                false => 0
            }
        }
    }

    /// set which indices will be avaiable for playing.
    /// 
    /// # examples
    /// 
    /// accepted values are:
    /// 
    /// * [std::ops::Range]
    /// ```
    ///     section.set_indices(0..2);
    /// ```
    /// * [std::ops::RangeInclusive]
    /// ```
    ///     [std::ops::Range]
    ///     section.set_indices(0..=1);
    /// ```
    /// 
    /// * iterators of type [(u32, u32)] where the first integer represents the `row`
    /// 
    ///     while the second represents the `comumn`:
    /// 
    /// ```
    ///     section.set_indices([(0,0),(1,0),(2,0),(3,0)]);
    /// 
    ///     section.set_indices(vec![(0,1),(1,1),(2,1)]);
    /// ```
    pub fn set_indices(&mut self, items: impl IntoIndices)
    {
        self.indices = items.into_indices(self.layout);
        self.index = match &self.indices
        {
            Some(tiles) => tiles.index as u32,
            None => 0
        }
    }
}

pub trait IntoIndices 
{
    /// converts an iteration to an array of indices
    /// aligned to the correct uv instance uvs
    fn into_indices(self,layout:SpriteLayout) -> Option<Tiles> ;
}

impl IntoIndices for std::ops::Range<u32>
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| (i*4) <= (layout.rows*layout.columns-1) *4)
            .collect:: <Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeInclusive<u32>
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| (i*4) <= (layout.rows*layout.columns-1) *4)
            .collect:: <Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeFull
{
    fn into_indices(self, _ : SpriteLayout) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for ()
{
    fn into_indices(self, _ :SpriteLayout) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for &[u32]
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.iter()
            .copied()
            .filter(|i| (i*4) <= (layout.rows*layout.columns-1) *4)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for &[(u32,u32)]
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.iter()
            .map(|(row,column)|column*layout.rows+row)
            .filter(|i|(i*4)<=(layout.rows*layout.columns-1)*4)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles {index:0,indices}),
            false => None
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod,bytemuck::Zeroable,Clone,Copy)]
struct SpriteInstanceRaw
{
    transform: crate::TransformRaw,
    index: u32
}

/// impl containing sprite loading 
impl SpriteBinding
{
    /// loads a [`SpriteSheetBinding`] from a [crate::SpriteLoader]
    ///
    /// # Panics
    ///
    /// Panics if the path is not found
    pub fn from_loader<T>(loader: crate::SpriteLoader<T>) -> Self
        where
            T: Into<std::ffi::OsString> + AsRef<std::path::Path>
    {
        let (ref id, filtermode, instances, pxunit, layout ) = match loader
        {
            crate::SpriteLoader::Sprite { path, filtermode, mut instances, pxunit } =>
            {
                let mut _instances = Vec::with_capacity(instances.len());

                for _ in 0..instances.len()
                {
                    _instances.push(SpriteInstance
                    {
                        transform: instances.pop().unwrap(),
                        section: SheetSection
                        {
                            layout: SpriteLayout{ rows: 1, columns: 1 },
                            indices: None,
                            index: 0,
                        }
                    })
                };

                (path, filtermode, _instances, pxunit, SpriteLayout::default())
            }
            crate::SpriteLoader::SpriteSheet { path, filtermode, layout, mut instances, pxunit } =>
            {
                let mut _instances = Vec::with_capacity(instances.len());

                for _ in 0..instances.len()
                {
                    let (transform, indices) = instances.pop().unwrap();

                    _instances.push
                    (
                        SpriteInstance
                        {
                            transform,
                            section:
                            {
                                indices
                                    .into_indices(layout)
                                    .map_or_else(|| SheetSection
                                {
                                    layout,
                                    index: 0,
                                    indices: None
                                }, |indices| SheetSection
                                {
                                    layout,
                                    index: 0,
                                    indices: Some(indices)
                                })
                            }
                        }
                    )
                };

                (path, filtermode, _instances, pxunit, layout )
            }
        };

        let image = image::io::Reader::open(id)
            .unwrap()
            .decode()
            .expect("failed to decode image, unsupported format");

        // if we need to rescale we need to rescale the dyn image and not this variables
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

        let texture = crate::create_texture
        (
            wgpu::TextureDescriptor
            {
                size,
                // we use the directory of the sprite we loaded
                // as a debug label after a monstrous conversion
                label: Some(id.as_ref().to_string_lossy().as_ref()),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[]
            }
        );

        crate::write_texture
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

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = crate::create_sampler
        (
            wgpu::SamplerDescriptor
            {
                address_mode_u: wgpu::AddressMode::MirrorRepeat,
                address_mode_v: wgpu::AddressMode::MirrorRepeat,
                address_mode_w: wgpu::AddressMode::MirrorRepeat,
                mag_filter: filtermode,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        );

        // we adjust the dimensions of the vertex positions using the 
        // pixel per unit factor
        let scale = baguette_math::Vec2::new
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

        let uvs =
        {
            let mut uvs = Vec::with_capacity((layout.rows * layout.columns) as usize);

            for i in 0..layout.rows * layout.columns 
            {
                uvs.append(&mut 
                {
                    // vertices with uncropped uvs
                    // meaning that the entirety of the texture will be rendered
                    let mut uvs = vec!
                    [
                        [0., 0.],
                        [0., 1.],
                        [1., 1.],
                        [1., 0.]
                    ];

                    let pos =
                    (
                        (i % layout.columns) as f32,
                        (i / layout.columns) as f32
                    );

                    let rows = layout.rows as f32;
                    let columns = layout.columns as f32;

                    // here we actually crop into the correct row and column
                    for uv in uvs.iter_mut()
                    {
                        uv[0] = (uv[0] / columns) + pos.0 * (1. / columns);
                        uv[1] = (uv[1] / rows) + pos.1 * (1. / rows)
                    }
                    uvs
                })
            }
            uvs
        };

        let texture = crate::Texture { texture, view, sampler, pxunit };

        Self
        {
            binding: Box::new(SpriteGpuBinding::new
            (
                &instances, &vertices, &uvs, id.as_ref().as_os_str(), &texture
            )),
            instances,
            texture,
        }
        
    } 
}

impl SpriteBinding
{
    /// iters the instances and will update the gpu data afterward
    pub fn iter_instances_mut(&mut self) -> SpriteIterMut
    {
        SpriteIterMut
        {
            instances: (&mut self.instances).into(),
            instance_buffer: &self.binding.instance_buffer.0,
            index: 0,
        }
    }

    /// iters the instances immutably
    pub fn iter_instances(&self) -> std::slice::Iter<'_, SpriteInstance>
    {
        self.instances.iter()
    }
    
    /// returns the size of the texture
    pub fn size(&self) -> baguette_math::Vec2
    {
        self.texture.size()
    }

    pub fn add_instances(&mut self, mut new_instances: Vec<SpriteInstance>)
    {
        // we add the new instances
        self.instances.append(&mut new_instances);         

        // .. then recreate the buffer to update it with the added instance
        let data: Vec<SpriteInstanceRaw> = self.instances.iter()
            .map(|f| f.as_raw())
            .collect();

        crate::write_buffer(&self.binding.instance_buffer.0, &data)
    }
}

/// sprite gpu binding
pub(super) struct SpriteGpuBinding
{
    pub vertex_buffer: wgpu::Buffer,

    /// contains the buffer and the instance count
    pub instance_buffer: (wgpu::Buffer,u32),

    pub bindgroup: wgpu::BindGroup,

    pub id: Option<u32>
}

impl SpriteGpuBinding
{
    fn new
    (
        instances: &[SpriteInstance], vertices: &[[f32;2]], uvs: &[[f32;2]],
        id: &std::ffi::OsStr, texture : &crate::Texture
    ) -> Self
    {
        let uvs_storage_buffer = crate::create_buffer_init
        (
            wgpu::util::BufferInitDescriptor
            {
                label: Some("uvs storage buffer"),
                contents: bytemuck::cast_slice(uvs),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
        );

        Self
        {
            vertex_buffer: crate::create_buffer_init
            (
                wgpu::util::BufferInitDescriptor
                {
                    label: Some("vertex buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
                }
            ),
            instance_buffer:
            {
                let instances = instances
                    .iter()
                    .map(|instance| instance.as_raw())
                    .collect::<Vec<SpriteInstanceRaw>>();

                (
                    crate::create_buffer_init
                    (
                        wgpu::util::BufferInitDescriptor
                        {
                            label: Some("instances"),
                            contents: bytemuck::cast_slice(&instances),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
                        }
                    ),
                    instances.len().try_into().expect("expected a little less instances")
                )
            },
            bindgroup: crate::create_bindgroup(wgpu::BindGroupDescriptor
            {
                label: Some(&("sprite sheet bindgroup, id: ".to_owned() + &id.to_string_lossy())),
                layout: &bindgroup_layout(),
                entries: 
                &[
                    wgpu::BindGroupEntry
                    {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view)
                    },
                    wgpu::BindGroupEntry
                    {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler)
                    },
                    wgpu::BindGroupEntry
                    {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer
                        (
                            uvs_storage_buffer.as_entire_buffer_binding()
                        )
                    }
                ]
            }),
            id: Some(baguette_math::rand::u32(..))
        }
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

#[must_use]
/// runtime instance of a sprite, contains both the texture and all the instances
pub struct Sprite
{
    pub(crate) sprite: SpriteBinding,
    /// this is only used on drop to remove the reference to this [Sprite]
    pub(crate) spritebuffer: std::ptr::NonNull<Vec<std::ptr::NonNull<SpriteGpuBinding>>>,
}

impl std::ops::Deref for Sprite
{
    type Target = SpriteBinding;

    fn deref(&self) -> &Self::Target
    {
        &self.sprite
    }
}

impl std::ops::DerefMut for Sprite
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.sprite
    }
}

impl Drop for Sprite
{
    fn drop(&mut self)
    {
        if let Some(id) = self.binding.id
        {
            unsafe { self.spritebuffer.as_mut().retain
            (
                |sprite| sprite.as_ref().id.unwrap() != id
            ) }  
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
        self.iter_instances_mut().nth(index).unwrap()
    }
}

pub struct SpriteIterMut<'a>
{
    instances: core::ptr::NonNull<Vec<SpriteInstance>>,
    instance_buffer: &'a wgpu::Buffer,

    index: usize
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
                .get_mut(self.index)
        };
        
        self.index += 1;

        next
    }
}

// we use the drop trait to update the instance buffer,
// we assume that there is no mutation after dropping the iterator
// so this should be the place to do it
impl Drop for SpriteIterMut<'_>
{
    fn drop(&mut self)
    {
        // SAFETY: we iter the slice reading the values, but we leave the mutation of the buffer to wgpu
        {
            let instances = unsafe { self.instances.as_ref() }
                .iter()
                .map(SpriteInstance::as_raw)
                .collect::<Vec<SpriteInstanceRaw>>();

            crate::write_buffer(self.instance_buffer, &instances);
        }
    }
}

/// costant value describing the indices required to render a 2d sprite
pub(crate) const SPRITE_INDICES: [u16; 6] =
[
    0, 1, 2, 2, 3, 0
];

/// the bindgroup layout of the sprite
pub(super) fn bindgroup_layout() -> wgpu::BindGroupLayout
{
    crate::create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor 
    {
        entries:
        &[
            wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture 
                {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true }
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry 
            {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None
            },
            wgpu::BindGroupLayoutEntry 
            {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer
                {
                    ty: wgpu::BufferBindingType::Storage{ read_only: true },
                    has_dynamic_offset: false, min_binding_size: None
                },
                count: None
            }
        ],
            label: Some("sprite layout")
        }
    )
}