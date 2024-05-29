use wgpu::*;
use crate::*;

pub struct TilemapPass
{
    tiles: Vec<Tile>,
    binding: TilemapBinding,
}

impl TilemapPass
{
    pub fn new(ctx: crate::ContextHandle) -> Self
    {
        Self
        {
            tiles: vec![],
            binding: TilemapBinding::new(ctx),
        }
    }
}

impl crate::RenderPass for TilemapPass
{
    fn add_pass(ctx: ContextHandle) -> Passes
    {
        Passes::Tilemap(TilemapPass::new(ctx))
    }

    fn draw<'a>
    (
        &'a mut self,
        _: &ContextHandleData,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData

    ) -> Result<(), SurfaceError>
    {
        pass.set_pipeline(&self.binding.pipeline);

        pass.set_bind_group(0, &camera.bindings.bindgroup, &[]);
        pass.set_bind_group(1, &self.binding.bindgroup, &[]);

        pass.set_vertex_buffer(0, self.binding.vert_buffer.slice(..));
        pass.set_vertex_buffer(1, self.binding.instance_buffer.slice(..));

        pass.set_index_buffer(self.binding.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        pass.draw_indexed(0..SPRITE_INDICES_U16.len() as _, 0, 0..1);
        
        Ok(())
    }
}

pub struct Tile
{
    pos: Vec2,
    uv: Vec2
}

pub struct TilemapBinding
{
    pipeline: RenderPipeline,
    bindgroup: BindGroup,
    textures: Vec<TextureData>,
    vert_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Buffer,
    mat_buffer: Buffer
}

impl TilemapBinding
{
    pub fn new(ctx: ContextHandle) -> Self
    {
        let ctx = ctx.data.read().unwrap();
        
        let textures = vec!
        [
            TextureData::from_bytes
            (
                &ctx.device, &ctx.queue, include_bytes!(r"D:\dev\Rust\baguette\assets\green dude.png"), "label"
            ).unwrap()
        ];

        let matrix = Mat4::IDENTITY.to_cols_array_2d();

        let mat_buffer = ctx.create_buffer_init(wgpu::util::BufferInitDescriptor
        {
            label: Some("tilemap matrix uniform"),
            contents: bytemuck::cast_slice(&matrix),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let (width,height) = ctx.screen.size();

        let layers_texture = ctx.create_texture(TextureDescriptor
        {
            label: None,
            size: Extent3d
            {
                width: std::cmp::max(width, 1),
                height: std::cmp::max(height, 1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Uint,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[],   
        }).create_view(&Default::default());

        let pipe_layout = ctx.create_pipeline_layout
        (
            PipelineLayoutDescriptor
            {
                label: Some(&format!("tilemap pipeline layout, {} textures", textures.len())),
                bind_group_layouts: 
                &[
                    &crate::camera_bindgroup_layout(&ctx),
                    &Self::tilemap_bindgroup_layout(&ctx, textures.len() as _)
                ],
                push_constant_ranges: &[]
            }
        );

        let shader = ctx.create_shader_module(include_wgsl!("tilemap.wgsl"));
        
        let pipeline = ctx.create_render_pipeline(RenderPipelineDescriptor
        {
            label: Some(&format!("tilemap pipeline, {} textures", textures.len())),
            layout: Some(&pipe_layout),
            vertex: VertexState
            {
                module: &shader,
                entry_point: "vertex",
                buffers:
                &[
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32x2],
                    },
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<[[f32; 2]; 2]>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &vertex_attr_array![1 => Float32x2, 2 => Float32x2]
                    }
                ],
            },
            fragment: Some(FragmentState
            {
                module: &shader,
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });
        
        let vertices: [[f32; 2]; 4] =
        [
            [-1., 1.],
            [-1., -1.],
            [1., -1.],
            [1., 1.]
        ];

        use wgpu::util::BufferInitDescriptor;
        let vert_buffer = ctx.create_buffer_init(BufferInitDescriptor
        {
            label: Some("tilemap vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = ctx.create_buffer_init(BufferInitDescriptor
        {
            label: Some("tilemap index buffer"),
            contents: bytemuck::cast_slice(&SPRITE_INDICES_U16),
            usage: BufferUsages::INDEX,
        });

        let instance: [[f32; 2]; 2] = [[0.,0.], [0.,1.]];

        let instance_buffer = ctx.create_buffer_init(BufferInitDescriptor
        {
            label: Some("tilemap vertex buffer"),
            contents: bytemuck::cast_slice(&instance),
            usage: BufferUsages::VERTEX,
        });

        Self
        {
            bindgroup: Self::tilemap_bindgroup(&ctx, &textures, &mat_buffer, &layers_texture),
            textures,
            mat_buffer,
            pipeline,
            vert_buffer,
            instance_buffer,
            index_buffer,
        }
    }

    fn tilemap_bindgroup
    (
        ctx: &ContextHandleData,
        textures: &[TextureData],
        matrix_buffer: &Buffer,
        layer_texture: &TextureView
    ) -> BindGroup
    {
        let views = textures.iter().map(|data| &data.view).collect::<Vec<_>>();
        let samplers = textures.iter().map(|data| &data.sampler).collect::<Vec<_>>();

        ctx.create_bindgroup(BindGroupDescriptor
        {
            label: Some
            (
                &format!("tilemap bindgroup with {} textures", views.len())
            ),
            layout: &Self::tilemap_bindgroup_layout(ctx, textures.len() as _),
            entries:
            &[
                BindGroupEntry
                {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&views)
                },
                BindGroupEntry
                {
                    binding: 1,
                    resource: BindingResource::SamplerArray(&samplers)
                },
                BindGroupEntry
                {
                    binding: 2,
                    resource: matrix_buffer.as_entire_binding()
                },
                BindGroupEntry
                {
                    binding: 3,
                    resource: BindingResource::TextureView(layer_texture)
                }
            ]
        })
    }

    fn tilemap_bindgroup_layout(ctx: &ContextHandleData, count: u32) -> BindGroupLayout
    {
        let count = std::num::NonZeroU32::new(count);

        ctx.create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor
        {
            label: Some("tilemap bind layout"),
            entries:
            &[
                BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture
                    {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count,
                },
                BindGroupLayoutEntry
                {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count,
                },
                BindGroupLayoutEntry
                {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer
                    {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                BindGroupLayoutEntry
                {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture
                    {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::R32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None
                }
            ]
        })
    }
}