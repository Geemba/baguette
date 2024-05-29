use std::{ffi::OsString, ptr::NonNull, sync::RwLockReadGuard};

use crate::*;
use sprite::*;

/// a starting value for how many sprite the instance buffer could hold
const SPRITE_INSTANCES_INITIAL_CAPACITY: usize = 50;

pub struct SpritePass
{
    sprites: Vec<NonNull<SpriteImpl>>,
    instances: Vec<SpriteInstanceRaw>,
    bindings: Option<SpriteBinding>
}

impl SpritePass
{
    pub fn new() -> Self
    {
        Self
        {
            sprites: vec![],
            instances: vec![],
            bindings: None,
        }
    }
 
    /// adds a new sprite to render, if another costructed [Sprite] has all the same parameters and texture
    /// you should add a new [SpriteInstance] inside of that struct using the instance parameter
    pub fn add_sprite(&mut self, ctx: crate::ContextHandle, loader: SpriteLoader) -> Sprite
    {
        //use wgpu::*;
        //use wgpu::util::BufferInitDescriptor;

        let ctx = ctx.read().expect("aw heel naw you panicked");
        
        let mut sprite = Box::new(SpriteImpl::from_loader(&ctx, loader));

        self.sprites.push
        (
            (&mut *sprite).into()
        );

        self.update_bindings(&ctx);

        Sprite
        {
            sprite,
            sprites: (&mut self.sprites).into()
        }
    }

    fn update_bindings(&mut self, ctx: &RwLockReadGuard<ContextHandleData>)
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
}

impl Default for SpritePass
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl RenderPass for SpritePass
{
    fn draw<'a>
    (
        &'a mut self, 
        ctx: &ContextHandleData,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a camera::CameraData
        
    )
    -> Result<(), wgpu::SurfaceError>
    {
        // instance sorting
        {
            self.instances.clear();
        
            for (i, sprite) in self.sprites.iter().enumerate()
            {
                let sprite = unsafe { sprite.as_ref() };

                for instance in &sprite.instances
                {
                    self.instances.push(instance.as_raw(&sprite.slice, sprite.pivot, i as u32))
                }
            }
            
            self.instances.sort_unstable_by
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

                    //println!
                    //(
                    //    "instance sorting point {}, other sorting point: {}",
                    //    instance_y_pos + instance_pivot,
                    //    other_y_pos + other_pivot
                    //);

                    f32::total_cmp
                    (
                        &(instance_y_pos + instance_pivot), 
                        &(other_y_pos + other_pivot) 
                    ).reverse()
                }
            );
        }
        
        let bindings = self.bindings.as_ref().unwrap();
        ctx.write_buffer(&bindings.instance_buffer, &self.instances);

        // drawing
        {
            pass.set_pipeline(&bindings.render_pipeline);

            pass.set_bind_group(0, &camera.bindings.bindgroup, &[]);
            pass.set_bind_group(1, &bindings.bindgroup, &[]);
            
            pass.set_vertex_buffer(0, bindings.index_buffer.slice(..));
            pass.set_vertex_buffer(1, bindings.instance_buffer.slice(..));

            pass.draw(0..SPRITE_INDICES_U32.len() as u32, 0..self.instances.len() as u32)
        }

        //for binding in self.sprites.values()
        //{
        //    let binding = unsafe { binding.as_ref() };

        //    pass.set_bind_group(0, &binding.binding.bindgroup, &[]);
        //    pass.set_bind_group(1, &camera.binding.bindgroup, &[]);
    
        //    pass.set_vertex_buffer(0, binding.binding.vertex_buffer.slice(..));
        //    pass.set_vertex_buffer(1, binding.binding.instance_buffer.0.slice(..));

        //    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //    pass.draw_indexed(0..SPRITE_INDICES.len() as u32, 0, 0..binding.binding.instance_buffer.1);
        //}

        Ok(())
    }

    fn add_pass(_: ContextHandle) -> Passes where Self: Sized
    {
        Passes::SpriteSheet(Self::new())
    }
}

/// describes the type of sprite you want to create
pub struct SpriteLoader
{
    /// directory of the source
    pub(crate) path: OsString,
    /// describes how the sprite will be filtered,
    /// 
    /// [wgpu::FilterMode::Nearest] results in a pixelated effect
    /// while [wgpu::FilterMode::Linear] makes textures smooth but blurry
    pub(crate) filtermode: wgpu::FilterMode,
    /// the pivot of the sprite, defaults to 0,0 (the center of the sprite) if [None].
    /// is used both as pivot of rotation and as sorting point with other sprites
    pub(crate) pivot: Option<Vec2>,
    pub(crate) instances: Vec<SpriteInstance>,
    /// describes how many pixels of this sprite
    /// represent one unit in the scene
    pub(crate) pxunit: f32,

    pub(crate) rows: u32, 
    pub(crate) columns: u32, 
}

impl SpriteLoader
{
    pub fn new(path: impl Into<std::ffi::OsString>) -> Self
    {
        Self
        {
            path: path.into(),
            filtermode: FilterMode::Linear,
            pivot: None,
            instances: vec![Default::default()],
            pxunit: 100.,
            rows: 1,
            columns: 1,
        }
    }

    pub fn new_pixelated(path: impl Into<std::ffi::OsString>) -> Self
    {
        let mut loader = Self::new(path);
        loader.filtermode = FilterMode::Nearest;
        loader
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

    pub fn instances(mut self, instances: impl IntoIterator<Item = SpriteInstance>) -> Self
    {
        let instances = instances
            .into_iter()
            .collect::<Vec<_>>();

        if !instances.is_empty()
        {
            self.instances = instances;
        }
        
        self
    }

    /// if this is an atlas, pass many rows and columns it has
    pub fn slice_atlas(mut self, rows: u32, columns: u32) -> Self
    {
        self.rows = u32::max(1, rows);
        self.columns = u32::max(1, columns);
        self
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

#[repr(C)]
#[derive(bytemuck::NoUninit, Clone, Copy)]
pub(crate) struct SpriteInstanceRaw
{
    pub transform: crate::TransformRaw,
    
    pub uv_idx: u32,
    pub bind_idx: u32,
}

/// handles to the gpu
pub(super) struct SpriteBinding
{
    pub shader: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bindgroup: wgpu::BindGroup,

    pub index_buffer: wgpu::Buffer,
    pub sprite_slices_storage_buffer: wgpu::Buffer,
    pub uv_uniform: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
}

impl SpriteBinding
{
    fn new
    (
        ctx: &RwLockReadGuard<ContextHandleData>,

        instances_capacity: usize,

        textures: &[&wgpu::TextureView],
        samplers: &[&wgpu::Sampler],
        sprite_slices: &[SpriteSlice],

    ) -> SpriteBinding
    {
        use wgpu::*;
        use wgpu::util::BufferInitDescriptor;

        assert!
        (
            textures.len() == samplers.len(),
            "unexpected amount of textures and samplers,
            both of them should have had equal length"
        );

        let sprite_slices_storage_buffer = ctx.create_buffer(BufferDescriptor
        {
            label:  Some("sprites slices storage buffer"),
            size: (std::mem::size_of::<SpriteSlice>() * 2) as u64,
            usage:  BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ctx.write_buffer(&sprite_slices_storage_buffer, sprite_slices);

        let uvs: [[f32; 4]; 4] =
        [
            [0.,0., /*<- data, */ 0., 0. /* <- padding */],
            [0.,1., /*<- data, */ 0., 0. /* <- padding */],
            [1.,1., /*<- data, */ 0., 0. /* <- padding */],
            [1.,0., /*<- data, */ 0., 0. /* <- padding */],
        ];

        let uv_uniform = ctx.create_buffer_init(BufferInitDescriptor
        {
            label: Some("sprite uv buffer"),
            contents: bytemuck::cast_slice(&uvs),
            usage: BufferUsages::UNIFORM
        });

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
            index_buffer: ctx.create_buffer_init(BufferInitDescriptor
            {
                label: Some("sprite index buffer"),
                contents: bytemuck::cast_slice(&SPRITE_INDICES_U32),
                usage: BufferUsages::VERTEX,
            }),
            sprite_slices_storage_buffer,
            instance_buffer: ctx.create_buffer(BufferDescriptor
            {
                label: Some("instance buffer"),
                size: (std::mem::size_of::<SpriteInstanceRaw>() * instances_capacity) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            uv_uniform,
            render_pipeline: Self::create_pipeline(ctx, &shader, textures.len()),
            shader,
        }
    }

    pub(crate) fn update
    (
        &mut self,
        ctx: &RwLockReadGuard<ContextHandleData>,

        textures: &[&wgpu::TextureView],
        samplers: &[&wgpu::Sampler],
        sprite_slices: &[SpriteSlice],
    )
    {
        ctx.write_buffer(&self.sprite_slices_storage_buffer, sprite_slices);

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
        ctx: &RwLockReadGuard<ContextHandleData>,
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
        ctx: &RwLockReadGuard<ContextHandleData>,
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
                ]
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
                ]
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

fn bindgroup_layout(ctx: &RwLockReadGuard<ContextHandleData>, count: usize) -> wgpu::BindGroupLayout
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