use std::vec;
use wgpu::*;

use crate::*;

/// vec containing different buffers, each sprite with a different source will create a new pass 
type SpriteBuffers = Vec<SpriteBuffer>;

/// a buffer containing data for rendering instances of a sprite
struct SpriteBuffer
{
    sprite : super::Sprite,
    vertex_buffer : Buffer,
    index_buffer : Buffer,
    instance_buffer : Buffer,
    bindgroup : BindGroup,
}

impl SpriteBuffer
{    
    fn new(sprite: super::Sprite, layout : &BindGroupLayout) -> Self
    {
        Self
        {
            vertex_buffer: sprite.mesh.create_vertex_buffer(),
            index_buffer: sprite.mesh.create_index_buffer(),
            instance_buffer: sprite.mesh.create_transform_buffer(),

            bindgroup: create_bindgroup(wgpu::BindGroupDescriptor
            {
                label: Some("sprite_bind_group"),
                layout,
                entries:
                &[
                    wgpu::BindGroupEntry
                    {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&sprite.texture.view),
                    },
                    wgpu::BindGroupEntry
                    {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sprite.texture.sampler),
                    }
                ],
            }),
            sprite,
        }
    }

    fn draw<'a>(&'a self, render_pass : &mut wgpu::RenderPass<'a>, camera_bind_group : &'a BindGroup)
    {
        render_pass.set_bind_group(0, &self.bindgroup, &[]);
        render_pass.set_bind_group(1, &camera_bind_group, &[]);  

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.sprite.mesh.indices.len() as u32, 0, 0..self.sprite.mesh.instances.len() as u32);  
    }

    fn add_instance(&mut self)
    {
        self.sprite.mesh.instances.push(Transform::default().to_raw());
        // we recreate the buffer to update it with the added instance
        self.instance_buffer = self.sprite.mesh.create_transform_buffer();
    }

}

/// a renderpass type specifically for sprites
pub struct SpritePass<'a>
{
    render_pipeline : RenderPipeline,
    camera_bind_group : &'a BindGroup,

    layout : BindGroupLayout,

    buffers : SpriteBuffers,
}

impl SpritePass<'_>
{
    pub fn init
    (
        cam : Option<&Camera>,
        backface_culling : bool
    ) -> Self
    {
        let cam = unsafe { std::mem::transmute::<&Camera, &Camera>(cam.unwrap_or(Camera::main())) };

        let config = super::config();

        let shader = create_shader_module(ShaderModuleDescriptor 
        {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(r"D:\Fruit_Dungeon\baguette\crates\baguette-rendering\src\rendering\shaders\shader.wgsl").into())
        });

        let sprite_layout = create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor 
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                    wgpu::BindGroupLayoutEntry 
                    {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
            ],
                label: Some("sprite layout"),
            }
        );

        let pipeline_layout = create_pipeline_layout(PipelineLayoutDescriptor
        {
            label: Some("2d pipeline layout"),
            bind_group_layouts:
            &[
                &sprite_layout,
                &cam.binding.layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(wgpu::RenderPipelineDescriptor 
        {
            label: Some("2d pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState
            {
                module : &shader,
                entry_point : "vertex_entry",
                buffers : &[super::vertex::vertex_layout_desc(), super::transform::desc()]
            },
            fragment: Some(wgpu::FragmentState // 3.
            {
                module: &shader,
                entry_point: "fragment_entry",
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
            primitive: wgpu::PrimitiveState
            {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: match backface_culling 
                {
                    true => Some(Face::Back),
                    false => None,
                },
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let mut sprite_pass = Self
        {
            layout: sprite_layout,
            render_pipeline,
            camera_bind_group : &cam.binding.bind_group,
            buffers: vec![],
        };

        sprite_pass.add(
        {
            let mut sprite = Sprite::from_path
            (
                r"D:\Fruit_Dungeon\baguette\assets\sprites\test.png",
                FilterMode::Linear, None 
            ).unwrap();

            sprite.mesh.instances = vec!
            [ 
                super::Transform::new
                (
                    super::TransformDescriptor
                    {
                        position: math::Vec3::NEG_Z, ..Default::default()
                    }
                ).to_raw()
            ];

            sprite
        });
        sprite_pass
    }

    pub fn add(&mut self, sprite: super::Sprite)
    {
        match self.buffers.iter_mut().find(|sprite_buffer| sprite_buffer.sprite == sprite)
        {
            Some(sprite_buffer) => sprite_buffer.add_instance(),
            None => self.buffers.push(SpriteBuffer::new(sprite, &self.layout)),
        }
    }
}

impl super::RenderPass for SpritePass<'_>
{
    fn draw
    (
        &self, encoder : &mut CommandEncoder, view : &TextureView
    ) -> Result<(), wgpu::SurfaceError>
    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor
        {
            label: Some("2d render pass"),
            color_attachments: &[Some(RenderPassColorAttachment
            {
                view : &view,
                resolve_target: None,
                ops: Operations
                {
                    load: LoadOp::Clear(Color
                    {
                        r: 0.2,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        
        render_pass.set_pipeline(&self.render_pipeline);
        
        //self.post_process
        for buffer in &self.buffers
        {
            buffer.draw(&mut render_pass, &self.camera_bind_group);
        }

        Ok(())
    }
}