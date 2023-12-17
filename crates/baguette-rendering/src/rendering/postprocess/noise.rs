//use crate::*;
//use wgpu::util::DeviceExt;
//
//pub struct NoisePostProcess
//{
//    pipeline : wgpu::RenderPipeline,
//    bind_group : wgpu::BindGroup,
//    parameters : NoiseParameters,
//}
//
//struct NoiseParameters
//{
//    seed_uniform : wgpu::Buffer,
//    brightness_uniform : wgpu::Buffer,
//    opacity_uniform : wgpu::Buffer,
//}
//
//impl NoisePostProcess
//{
//    pub fn new(brightness : f32, opacity : f32) -> Self
//    {
//        let bind_group_layout = Self::bind_layout();
//
//        // uniform creation
//
//            let time = std::time::Instant::now();
//
//            let seed_uniform = device().create_buffer_init(&wgpu::util::BufferInitDescriptor
//            {
//                label: Some("seed uniform"),
//                contents: bytemuck::cast_slice(&[time.elapsed().as_secs_f32()]),
//                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//            });
//
//            let opacity_uniform = device().create_buffer_init(&wgpu::util::BufferInitDescriptor
//            {
//                label: Some("opacity uniform"),
//                contents: bytemuck::cast_slice(&[opacity]),
//                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//            });
//
//            let brightness_uniform = device().create_buffer_init(&wgpu::util::BufferInitDescriptor
//            {
//                label: Some("brightness uniform"),
//                contents: bytemuck::cast_slice(&[brightness]),
//                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//            });
//
//        //
//
//        let bind_group = create_bindgroup(wgpu::BindGroupDescriptor 
//        {
//            label: Some("noise bindgroup"),
//            layout: &Self::bind_layout(),
//            entries:
//            &[wgpu::BindGroupEntry
//            {
//                binding: 0,
//                resource: seed_uniform.as_entire_binding(),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 1,
//                resource: opacity_uniform.as_entire_binding(),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 2,
//                resource: brightness_uniform.as_entire_binding(),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 3,
//                resource: wgpu::BindingResource::TextureView(postprocess::PostProcessPasses::processing_view()),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 4,
//                resource: wgpu::BindingResource::Sampler(postprocess::PostProcessPasses::sampler()),
//            }]
//        });
//
//        let shader = create_shader_module(wgpu::ShaderModuleDescriptor
//        {
//            label: Some("noise shader"),
//            source: wgpu::ShaderSource::Wgsl(include_str!(r"D:\Fruit_Dungeon\baguette\crates\baguette-rendering\src\rendering\postprocess\noise.rs").into()),
//        });
//
//        let pipeline = create_render_pipeline(wgpu::RenderPipelineDescriptor
//        {
//            label: Some("noise resolution pipeline"),
//
//            layout: Some(&create_pipeline_layout(wgpu::PipelineLayoutDescriptor
//            {
//                label: Some("noise pipeline layout"),
//                bind_group_layouts: &[&bind_group_layout],
//                push_constant_ranges: &[],
//            })),
//            vertex: wgpu::VertexState
//            {
//                module: &shader,
//                entry_point: "vertex",
//                buffers: &[crate::vertex_layout_desc()],
//            },
//            fragment: Some(wgpu::FragmentState
//            {
//                module: &shader,
//                entry_point: "noise_grayscale",
//                targets: &[Some(wgpu::ColorTargetState
//                {
//                    format: config().format,
//                    blend: Some(wgpu::BlendState
//                    {
//                        color: wgpu::BlendComponent::REPLACE,
//                        alpha: wgpu::BlendComponent::REPLACE,
//                    }),
//                    write_mask: wgpu::ColorWrites::ALL,
//                })],
//            }),
//            primitive: wgpu::PrimitiveState::default(),
//            multisample: wgpu::MultisampleState::default(),
//            multiview: Default::default(),
//            depth_stencil: None,
//        });
//
//        Self 
//        {
//            pipeline,
//            bind_group,
//            parameters : NoiseParameters     
//            {
//                seed_uniform,
//                brightness_uniform,
//                opacity_uniform,
//            },
//        }
//    }
//
//    fn bind_layout() -> wgpu::BindGroupLayout
//    {
//        create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor
//        {
//            label: Some("noise bindgroup layout"),
//            entries: &[
//            wgpu::BindGroupLayoutEntry
//            {
//                binding: 0,
//                visibility: wgpu::ShaderStages::FRAGMENT,
//                ty: wgpu::BindingType::Buffer
//                { 
//                    ty: wgpu::BufferBindingType::Uniform,
//                    has_dynamic_offset: false,
//                    min_binding_size: None
//                },
//                count: None,
//            },
//            wgpu::BindGroupLayoutEntry
//            {
//                binding: 1,
//                visibility: wgpu::ShaderStages::FRAGMENT,
//                ty: wgpu::BindingType::Buffer 
//                { 
//                    ty: wgpu::BufferBindingType::Uniform,
//                    has_dynamic_offset: false,
//                    min_binding_size: None
//                },
//                count: None,
//            },
//            wgpu::BindGroupLayoutEntry
//            {
//                binding: 2,
//                visibility: wgpu::ShaderStages::FRAGMENT,
//                ty: wgpu::BindingType::Buffer 
//                { 
//                    ty: wgpu::BufferBindingType::Uniform,
//                    has_dynamic_offset: false,
//                    min_binding_size: None
//                },
//                count: None,
//            },
//            wgpu::BindGroupLayoutEntry
//            {
//                binding: 4,
//                visibility: wgpu::ShaderStages::FRAGMENT,
//                ty: wgpu::BindingType::Texture
//                {
//                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                    view_dimension: wgpu::TextureViewDimension::D2,
//                    multisampled: false
//                },
//                count: None,
//            },
//            wgpu::BindGroupLayoutEntry
//            {
//                binding: 3,
//                visibility: wgpu::ShaderStages::FRAGMENT,
//                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
//                count: None,
//            }]
//        })
//    }
//
//    /// Sets the brightness of this noise to a value between 0 and 1
//    pub fn set_brightness(&self, value : f32)
//    {
//        write_buffer(&self.parameters.brightness_uniform, &[value])  
//    }
//
//    /// Sets the opacity of this noise to a value between 0 and 1
//    pub fn set_opacity(&self, value : f32)
//    {
//        write_buffer(&self.parameters.opacity_uniform, &[value])  
//    }
//}
//
//impl crate::PostProcessPass for NoisePostProcess
//{
//    fn pass
//    (
//        &self, encoder : &mut wgpu::CommandEncoder,
//        view : &wgpu::TextureView, post_data : &PostProcessData
//    )
//    -> Result<(), wgpu::SurfaceError>
//    {
//        write_buffer(&self.parameters.seed_uniform, &[baguette_math::rand::f32()]);
//
//        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
//        {
//            label: Some("noise postprocess pass"),
//            color_attachments: &[Some(wgpu::RenderPassColorAttachment
//            {
//                view,
//                resolve_target: None,
//                ops: wgpu::Operations::default()
//            })],
//            depth_stencil_attachment: None,
//        });
//
//        pass.set_pipeline(&self.pipeline);
//        pass.set_bind_group(0, &self.bind_group, &[]);
//        pass.set_vertex_buffer(0, post_data.vertex_buffer.slice(..));
//        pass.set_index_buffer(post_data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
//        pass.draw_indexed(0..6, 0, 0..1);  
//
//        Ok(())
//    }
//
//    fn update_bindings(&mut self, view : &wgpu::TextureView, sampler : &wgpu::Sampler)
//    {
//        self.bind_group = create_bindgroup(wgpu::BindGroupDescriptor 
//        {
//            label: Some("noise bindgroup"),
//            layout: &Self::bind_layout(),
//            entries:
//            &[wgpu::BindGroupEntry
//            {
//                binding: 0,
//                resource: wgpu::BindingResource::TextureView(view),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 1,
//                resource: wgpu::BindingResource::Sampler(sampler),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 2,
//                resource: self.parameters.seed_uniform.as_entire_binding(),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 3,
//                resource: wgpu::BindingResource::TextureView(postprocess::PostProcessPasses::processing_view()),
//            },
//            wgpu::BindGroupEntry
//            {
//                binding: 4,
//                resource: wgpu::BindingResource::Sampler(postprocess::PostProcessPasses::sampler()),
//            }]
//        });
//    }
//}