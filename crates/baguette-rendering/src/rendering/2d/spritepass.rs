use crate::*;
use sprite::*;
use util::TBuffer;

use std::{ path::PathBuf, ptr::NonNull };

/// a starting value for how many sprite the instance buffer could hold
const SPRITE_INSTANCES_INITIAL_CAPACITY: usize = 50;

#[derive(Default)]
pub(crate) struct SpritePass
{
    sprites: Vec<NonNull<SpriteImpl>>,
    layers: FastIndexMap<u8,Vec<SpriteInstanceRaw>>,
    bindings: Option<SpriteBinding>
}

impl SpritePass
{
    pub fn add_sprite(&mut self, ctx: &ContextHandleInner, builder: SpriteBuilder) -> Sprite
    {   
        let mut sprite = Box::new(builder.build(ctx));

        self.sprites.push
        (
            (&mut *sprite).into()
        );

        self.update_bindings(ctx);

        Sprite
        {
            sprite,
            sprites: (&mut self.sprites).into()
        }
    }

    fn update_bindings(&mut self, ctx: &ContextHandleInner)
    {
        unsafe
        {
            let textures = self.sprites.iter().map(|sprite| &sprite.as_ref().texture.view).collect::<Vec<_>>();
            let samplers = self.sprites.iter().map(|sprite| &sprite.as_ref().texture.sampler).collect::<Vec<_>>();
            let sprite_slices = &self.sprites.iter().map(|sprite| sprite.as_ref().slice).collect::<Vec<_>>();

            match self.bindings
            {
                Some(ref mut binding) =>
                    binding.update(ctx, &textures, &samplers, sprite_slices),
                
                None => 
                {
                    self.bindings = Some(SpriteBinding::new
                    (
                        ctx,
                        SPRITE_INSTANCES_INITIAL_CAPACITY,
                        &textures, &samplers, sprite_slices
                    ));
                }
            }
        }
    }

    pub(crate) fn prepare_instances(&mut self)
    {
        self.layers.clear();
        
        for i in 0..self.sprites.len()
        {
            let sprite = unsafe { self.sprites[i].as_mut() };
            
            for (layer, new_instances) in sprite.layers.iter_mut()
            {
                let mut new_instances = new_instances.iter().map
                (
                    |instance| instance.as_raw(&sprite.slice, sprite.pivot, i as u32)
                ).collect();

                match self.layers.get_mut(layer)
                {
                    Some(instances) => instances.append(&mut new_instances),
                    None =>
                    {
                        self.layers.insert_sorted(*layer, new_instances);
                    }
                }
            }     
        }
        
        for instances in self.layers.values_mut()
        {
            instances.sort_unstable_by
            (
                |instance, other| unsafe
                {
                    // instance
            
                    let instance_pivot = self.sprites[instance.bind_idx as usize].as_ref().pivot.unwrap_or_default().y;
                        
                    let instance_y_pos = instance.transform[3][1]; // [3] is the translation, [1] is the y position
            
                    // other
            
                    let other_pivot = self.sprites[other.bind_idx as usize].as_ref().pivot.unwrap_or_default().y;
            
                    let other_y_pos = other.transform[3][1]; // [3] is the translation, [1] is the y position
            
                    //
            
                    f32::total_cmp
                    (
                        &(instance_y_pos + instance_pivot), 
                        &(other_y_pos + other_pivot) 
                    ).reverse()
                }
            )   
        }
    }

    pub(crate) fn draw<'a>
    (
        &'a self,
        ctx: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a camera::CameraData,
        layer: u8
    )
    {
        let instances = &self.layers[&layer];
        let bindings = self.bindings.as_ref().unwrap();
        ctx.write_entire_buffer(&bindings.instance_buffer, instances);

        pass.set_pipeline(&bindings.render_pipeline);

        pass.set_bind_group(0, &camera.bindings.bindgroup, &[]);
        pass.set_bind_group(1, &bindings.bindgroup, &[]);
        
        pass.set_vertex_buffer(0, bindings.index_buffer.slice(..));
        pass.set_vertex_buffer(1, bindings.instance_buffer.slice(..));

        pass.draw(0..SPRITE_INDICES_U32.len() as u32, 0..instances.len() as u32)  
    }
}

/// describes the type of sprite you want to create
pub struct SpriteBuilder
{
    /// directory of the source
    pub(crate) path: PathBuf,
    /// describes how the sprite will be filtered,
    /// 
    /// [wgpu::FilterMode::Nearest] results in a pixelated effect
    /// while [wgpu::FilterMode::Linear] makes textures smooth but blurry
    pub(crate) filtermode: Option<wgpu::FilterMode>,
    /// the pivot of the sprite, defaults to 0,0 (the center of the sprite) if [None].
    /// is used both as pivot of rotation and as sorting point with other sprites
    pub(crate) pivot: Option<Vec2>,
    pub(crate) instances: FastIndexMap<u8,Vec<SpriteInstance>>,
    /// describes how many pixels of this sprite
    /// represent one unit in the scene
    pub(crate) pxunit: f32,

    pub(crate) rows: u32, 
    pub(crate) columns: u32, 
}

impl SpriteBuilder
{
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self
    {

        let mut instances = FastIndexMap::default();
        instances.insert(0, vec![Default::default()]);

        Self
        {
            path: path.into(),
            filtermode: None,
            pivot: None,
            instances,
            pxunit: 100.,
            rows: 1,
            columns: 1,
        }
    }

    pub fn filter_mode(mut self, filter_mode: FilterMode) -> Self
    {
        self.filtermode = Some(filter_mode);
        self
    }

    pub fn pivot(mut self, pivot: impl Into<Vec2>) -> Self
    {
        let pivot = pivot.into();

        if pivot != Vec2::ZERO
        {
            self.pivot = Some(pivot);
        }
        
        self
    }

    /// Inserts the instances of this layer
    pub fn set_layer<const LAYER: u8>(mut self, instances: impl IntoIterator<Item = SpriteInstance>) -> Self
    {
        let instances = instances
            .into_iter()
            .collect::<Vec<_>>();

        if !instances.is_empty()
        {
            self.instances.insert(LAYER, instances);
        }
        
        self
    }

    /// if this is an atlas, pass many rows and columns it has
    pub fn tiled_atlas(mut self, rows: u32, columns: u32) -> Self
    {
        self.rows = u32::max(1, rows);
        self.columns = u32::max(1, columns);
        self
    }

        /// loads a [`SpriteBinding`] from a [crate::SpriteBuilder].
    ///
    /// panics if the path is not found
    pub fn build(self, ctx: &ContextHandleInner) -> SpriteImpl
    {
        let SpriteBuilder { ref path, filtermode, pivot, instances, pxunit, rows, columns } = self;

        let image = image::io::Reader::open(path)
            .unwrap()
            .decode()
            .expect("failed to decode image, unsupported format");

        // if we need to rescale we need to do it on the dyn image and not this variable
        // otherwhise we just crop the rendered texture
        let dimensions = Into::<UVec2>::into
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

        let filter_mode = filtermode.unwrap_or
        (
            match size.width / rows < 64 && size.height / columns < 64
            {
                true => FilterMode::Nearest,
                false => FilterMode::Linear
            }
        );

        let view = texture.create_view(&Default::default());
        let sampler = ctx.create_sampler
        (
            wgpu::SamplerDescriptor
            {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: filter_mode,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        // we adjust the dimensions of the vertex positions using the 
        // pixel per unit factor
        let scale = Vec2::new
        (
            (dimensions.x / columns) as f32 / pxunit,
            (dimensions.y / rows) as f32 / pxunit
        );

        let vertices =
        [
            [-scale.x, scale.y],
            [-scale.x, -scale.y],
            [scale.x, -scale.y],
            [scale.x, scale.y]
        ];

        let texture = crate::TextureData { texture, view, sampler };

        let slice = SpriteSlice::new(vertices, rows, columns);

        SpriteImpl
        {
            layers: instances,
            texture,
            pivot,
            slice,
        }
    }
}

#[repr(C)]
#[derive(bytemuck::NoUninit, Clone, Copy)]
pub(crate) struct SpriteSlice
{
    pub(crate) vertices: [[f32; 2]; 4],
    pub(crate) rows: u32,
    pub(crate) columns: u32,
    _padding: [f32; 2]
}

impl SpriteSlice
{
    pub(crate) fn new(vertices: [[f32; 2]; 4], rows: u32, columns: u32,) -> Self
    {
        Self
        {
            vertices,
            rows,
            columns,
            _padding: Default::default()
        }
    }
}

type Matrix = [[f32; 4]; 4];

#[repr(C)]
#[derive(bytemuck::NoUninit, Clone, Copy)]
pub(crate) struct SpriteInstanceRaw
{
    pub transform: Matrix,
    
    pub uv_idx: u32,
    pub bind_idx: u32,
}

/// the uvs of the spritepass are stored inside a storage array
/// which requires 16 byte align.
/// 
/// *(the last two floats are just padding)*
type Uv = [f32; 4];

/// handles to the gpu
pub(super) struct SpriteBinding
{
    pub shader: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bindgroup: wgpu::BindGroup,

    pub index_buffer: TBuffer<u32>,
    pub sprite_slices_storage_buffer: TBuffer<SpriteSlice>,
    pub uv_uniform: TBuffer<Uv>,
    pub instance_buffer: TBuffer<SpriteInstanceRaw>,
}

impl SpriteBinding
{
    fn new
    (
        ctx: &ContextHandleInner,

        instances_capacity: usize,

        textures: &[&wgpu::TextureView],
        samplers: &[&wgpu::Sampler],
        sprite_slices: &[SpriteSlice],

    ) -> SpriteBinding
    {
        use wgpu::*;

        assert!
        (
            textures.len() == samplers.len(),
            "unexpected amount of textures and samplers,
            both of them should have had equal length"
        );

        let sprite_slices_storage_buffer = ctx.create_buffer
        (
            Some("sprites slices storage buffer"),
            std::mem::size_of::<SpriteSlice>() * 2,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            false,
        );

        ctx.write_entire_buffer(&sprite_slices_storage_buffer, sprite_slices);

        let uvs: [[f32; 4]; 4] =
        [
            [0.,0., /*<- data, */ 0., 0. /* <- padding */],
            [0.,1., /*<- data, */ 0., 0. /* <- padding */],
            [1.,1., /*<- data, */ 0., 0. /* <- padding */],
            [1.,0., /*<- data, */ 0., 0. /* <- padding */],
        ];

        let uv_uniform = ctx.create_buffer_init
        (
            Some("sprite uv buffer"),
            bytemuck::cast_slice(&uvs),
            BufferUsages::UNIFORM
        );

        let shader = ctx.create_shader_module(wgpu::ShaderModuleDescriptor 
        {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(r"sprite.wgsl").into())
        });

        SpriteBinding
        {
            bindgroup: Self::create_bindgroup
            (
                ctx, textures, samplers, &sprite_slices_storage_buffer, sprite_slices, &uv_uniform
            ),

            index_buffer: ctx.create_buffer_init
            (
                Some("sprite index buffer"),
                &SPRITE_INDICES_U32,
                BufferUsages::VERTEX,
            ),

            sprite_slices_storage_buffer,
            instance_buffer: ctx.create_buffer
            (
                Some("instance buffer"),
                std::mem::size_of::<SpriteInstanceRaw>() * instances_capacity,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
                false,
            ),
            uv_uniform,
            render_pipeline: Self::create_pipeline(ctx, &shader, textures.len()),
            shader,
        }
    }

    pub(crate) fn update
    (
        &mut self,
        ctx: &ContextHandleInner,

        textures: &[&wgpu::TextureView],
        samplers: &[&wgpu::Sampler],
        sprite_slices: &[SpriteSlice],
    )
    {
        ctx.write_entire_buffer(&self.sprite_slices_storage_buffer, sprite_slices);

        self.bindgroup = Self::create_bindgroup
        (
            ctx,
            textures,
            samplers,
            &self.sprite_slices_storage_buffer,
            sprite_slices,
            &self.uv_uniform
        );

        self.render_pipeline = Self::create_pipeline(ctx, &self.shader, textures.len())
    }

    fn create_bindgroup
    (
        ctx: &ContextHandleInner,
        textures: &[&wgpu::TextureView],
        samplers: &[&wgpu::Sampler],
        sprite_slices_storage_buffer: &wgpu::Buffer,
        sprite_slices: &[SpriteSlice],
        uv_uniform: &wgpu::Buffer

    ) -> wgpu::BindGroup
    {
        use wgpu::*;
        
        assert!
        (
            textures.len() == samplers.len(),
            "unexpected amount of textures and samplers,
            both of them should have been equal"
        );

        ctx.create_bindgroup(BindGroupDescriptor
        {
            label: Some("sprite bindgroup"),
            layout: &bindgroup_layout(ctx, textures.len()),
            entries: &
            [
                BindGroupEntry
                {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(textures)
                },
                BindGroupEntry
                {
                    binding: 1,
                    resource: BindingResource::SamplerArray(samplers),
                },
                BindGroupEntry
                {
                    binding: 2,
                    resource: BindingResource::Buffer
                    (
                        BufferBinding 
                        {
                            buffer: sprite_slices_storage_buffer,
                            offset: 0,
                            size: Some
                            (
                                std::num::NonZeroU64::new
                                (
                                    std::mem::size_of_val(sprite_slices) as u64
                                ).unwrap()
                            ),
                        }
                    ),
                },
                BindGroupEntry
                {
                    binding: 3,
                    resource: uv_uniform.as_entire_binding()        
                }
            ],
        })
    }

    fn create_pipeline
    (
        ctx: &ContextHandleInner,
        shader: &wgpu::ShaderModule,
        count: usize
    )
    -> wgpu::RenderPipeline
    {
        use wgpu::*;

        let pipeline_layout = ctx.create_pipeline_layout(PipelineLayoutDescriptor
        {
            label: Some("2d sprite pipeline layout"),
            bind_group_layouts:
            &[
                &camera_bindgroup_layout(ctx),
                &bindgroup_layout(ctx, count)
            ],
            push_constant_ranges: &[]
        });
        
        ctx.create_render_pipeline(RenderPipelineDescriptor 
        {
            label: Some("2d sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState
            {
                module: shader,
                entry_point: "vertex",
                buffers:
                &[
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<u32>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Uint32],
                    },
                    // for each instance we pass the model matrix and an index
                    // that we'll use to get the uvs of the vertices from the storage buffer
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<([[f32; 4]; 4], u32, u32)>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &vertex_attr_array!
                        [
                            1 => Float32x4,
                            2 => Float32x4,
                            3 => Float32x4,
                            4 => Float32x4,
                            
                            5 => Uint32, // uv index
                            6 => Uint32, // bind index
                        ]
                    }
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState
            {
                module: shader,
                entry_point: "fragment",
                targets: 
                &[
                    Some(ColorTargetState
                    {
                        format: ctx.screen.config.format,
                        write_mask: ColorWrites::ALL,
                        blend: Some(BlendState::ALPHA_BLENDING)
                    })
                ],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState
            {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview: None
        })
    }
}

fn bindgroup_layout(ctx: &ContextHandleInner, count: usize) -> wgpu::BindGroupLayout
{
    use wgpu::*;
    use std::num::NonZeroU32;

    let count = NonZeroU32::new(count as u32);
    
    ctx.create_bindgroup_layout
    (
        BindGroupLayoutDescriptor
        {
            label: Some("sprite pass layout"),
            entries: &
            [
                BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture
                    {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false
                    },
                    count,
                },
                BindGroupLayoutEntry 
                {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty:  BindingType::Sampler(SamplerBindingType::Filtering),
                    count
                },
                BindGroupLayoutEntry 
                {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer
                    {
                        ty: BufferBindingType::Storage{ read_only: true },
                        has_dynamic_offset: false, min_binding_size: None
                    },
                    count: None
                },
                BindGroupLayoutEntry 
                {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer
                    {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false, min_binding_size: None
                    },
                    count: None
                }
            ],
        }
    )
}