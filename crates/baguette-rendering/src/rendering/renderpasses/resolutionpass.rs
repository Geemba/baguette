use wgpu::{*, util::BufferInitDescriptor};
use crate::*;

pub struct ResolutionPass
{
    pub render_pipeline : RenderPipeline,
    pub vertex_buffer : Buffer,
    pub index_buffer : Buffer,

    empty_pipeline : RenderPipeline,

    /// bindgroup and the associated
    /// view packed together for convenience
    low_res : LowResData
}

const SCREEN_VERTICES : [Vertex; 4] =
{
    [
        Vertex{ position: [-1., 1., 0.], tex_coords: [0., 0.] },
        Vertex{ position: [-1., -1., 0.], tex_coords: [0., 1.] },
        Vertex{ position: [1., -1., 0.], tex_coords: [1., 1.] },
        Vertex{ position: [1., 1., 0.], tex_coords: [1., 0.] },
    ]
};

struct LowResData
{
    view : TextureView,
    bind_group : BindGroup,
}

impl ResolutionPass
{
    fn create_low_res(layout : &BindGroupLayout, config : &SurfaceConfiguration) -> LowResData
    {
        let view = create_texture
        (
            TextureDescriptor
            {
                label: Some("low res texture"),
                size: Extent3d
                {
                    width: config.width / 2,
                    height: config.height / 2,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: config.format,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }
        ).create_view(&TextureViewDescriptor::default());

        let sampler = create_sampler(SamplerDescriptor
        {
            label: Some("low res sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = create_bindgroup(wgpu::BindGroupDescriptor
        {
            label: Some("sprite bind group"),
            layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }, 
            ],
        });

        LowResData
        {
            view,
            bind_group,
        }
    }

    pub fn init(config : &SurfaceConfiguration) -> Self
    {
        let vertex_buffer = create_buffer_init(BufferInitDescriptor
        {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&SCREEN_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let indices : std::vec::Vec::<u16> =
        vec!
        [
            0, 1 ,2 , 2, 3 ,0
        ];

        let index_buffer = create_buffer_init(BufferInitDescriptor
        {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let binding = std::env::current_dir().unwrap();
        let shaderpath = binding.display();

        println!("{shaderpath}");

        let shader = create_shader_module(ShaderModuleDescriptor
        {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(r"D:\Fruit_Dungeon\baguette\crates\baguette-rendering\src\rendering\postprocess\postprocess.rs").into())
        });

        let bindgroup_layout = create_bindgroup_layout
        (
            BindGroupLayoutDescriptor
            {
                label: Some("bind group layout"),
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
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry
                    {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            }
        );

        let pipeline_layout = create_pipeline_layout(PipelineLayoutDescriptor
        {
            label: Some("low resolution pipeline layout"),
            bind_group_layouts: &[&bindgroup_layout],
            push_constant_ranges: &[],
        });
    
        let render_pipeline = create_render_pipeline
        (
            RenderPipelineDescriptor
            {
                layout: Some(&pipeline_layout),
                label: Some("post process resolution pipeline"),

                vertex: VertexState
                {
                    module: &shader,
                    entry_point: "vertex",
                    buffers: &[vertex::vertex_layout_desc()],
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
            }
        );

        let empty_pipeline = create_render_pipeline
        (
            RenderPipelineDescriptor
            {
                layout: Some
                (
                    &create_pipeline_layout(wgpu::PipelineLayoutDescriptor 
                    {
                        label: None,
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    })
                ),
                label: Some("post process resolution pipeline"),

                vertex: VertexState
                {
                    module: &shader,
                    //this will create a triangle directly on shader without buffer
                    entry_point: "self_contained_vertex_test",
                    buffers: &[],
                },
                fragment: Some(FragmentState
                {
                    module: &shader,
                    entry_point: "fragment_test_color",
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
            }
        );

        let low_res =  Self::create_low_res(&bindgroup_layout, config);
           
        Self
        {
            render_pipeline,
            vertex_buffer,
            low_res,
            index_buffer,
            empty_pipeline,
        }
    }
}

impl super::RenderPass for ResolutionPass
{
    fn draw
    (
        &self, encoder : &mut CommandEncoder, view : &TextureView
    ) -> Result<(), wgpu::SurfaceError> 
    {
        {
            let mut low_res_pass = encoder.begin_render_pass(&RenderPassDescriptor 
            {
                label: Some("low res"),
                color_attachments: &[Some(RenderPassColorAttachment
                {
                    view : &self.low_res.view,
                    resolve_target: None,
                    ops: Operations
                    {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            low_res_pass.set_pipeline(&self.empty_pipeline);
            low_res_pass.draw(0..3, 0..1);

        }

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor 
        {
            label: Some("low res"),
            color_attachments: &[Some(RenderPassColorAttachment
            {
                view : &view,
                resolve_target: None,
                ops: Operations
                {
                    load: LoadOp::Clear(Color
                    {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.low_res.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);      

        Ok(())
    }
}