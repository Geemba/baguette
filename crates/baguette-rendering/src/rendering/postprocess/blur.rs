use wgpu::{*, util::*};
use crate::*;

pub struct Blur
{
    pipeline : wgpu::RenderPipeline,
    bind_group : wgpu::BindGroup,
    opacity : Buffer
}

impl Blur 
{
    pub fn new(config : &wgpu::SurfaceConfiguration, post : &PostProcessData, opacity : f32) -> Self
    {
        let layout = Self::create_layout();

        let shader = device().create_shader_module(ShaderModuleDescriptor
        {
            label: Some("noise shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(r"D:\Fruit_Dungeon\baguette\crates\baguette-rendering\src\rendering\shaders\blur.wgsl").into()),
        });

        let opacity = create_buffer_init(BufferInitDescriptor 
        {
            label: Some("blur opacity"),
            contents: bytemuck::cast_slice(&[opacity]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = create_bindgroup(BindGroupDescriptor 
        {
            label: Some("noise bindgroup"),
            layout: &layout,

            entries: &[wgpu::BindGroupEntry
            {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&post.processing),
            },
            wgpu::BindGroupEntry
            {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&post.sampler),
            },
            wgpu::BindGroupEntry
            {
                binding: 2,
                resource: opacity.as_entire_binding(),
            }]
        });

        let pipeline = create_render_pipeline(RenderPipelineDescriptor
        {
            label: Some("noise resolution pipeline"),

            layout: Some(&create_pipeline_layout(PipelineLayoutDescriptor
            {
                label: Some("noise pipeline layout"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            })),
            vertex: VertexState
            {
                module: &shader,
                entry_point: "vertex",
                buffers: &[super::vertex_layout_desc()],
            },
            fragment: Some(FragmentState
            {
                module: &shader,
                entry_point: "fragment",
                targets: &[Some(wgpu::ColorTargetState 
                {
                    format: config.format,
                    blend: Some(wgpu::BlendState 
                    {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            multiview: Default::default(),
            depth_stencil: None,
        });

        Self 
        {
            pipeline,
            bind_group,
            opacity,
        }

    }

    /// Sets the opacity of this [`Noise`] to a value between 0 and 1
    pub fn set_opacity(&self, value : f32)
    {
        queue().write_buffer(&self.opacity, 0, bytemuck::cast_slice(&[value]));    
    }

    fn create_layout() -> BindGroupLayout
    {
        device().create_bind_group_layout(&BindGroupLayoutDescriptor
        {
            label: Some("noise bindgroup layout"),
            entries: &[wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture
                {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer 
                { 
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }]
        })
    }
}

impl PostProcessPass for Blur
{
    fn pass
    (
        &self, encoder : &mut wgpu::CommandEncoder,
        view : &wgpu::TextureView, post : &PostProcessData
    )
    -> Result<(), wgpu::SurfaceError>
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor
        {
            label: Some("Noise Post Process Pass"),
            color_attachments: &[Some(RenderPassColorAttachment
            {
                view,
                resolve_target: None,
                ops: Operations
                {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, post.vertex_buffer.slice(..));
        pass.set_index_buffer(post.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..1);  

        Ok(())
    }

    fn update_bindings(&mut self, view : &TextureView, sampler : &Sampler)
    {
        let bind_group_layout = Self::create_layout();

        self.bind_group = device().create_bind_group(&BindGroupDescriptor 
        {
            label: Some("noise bindgroup"),
            layout: &bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: self.opacity.as_entire_binding(),
                }
            ]
        });
    }
}