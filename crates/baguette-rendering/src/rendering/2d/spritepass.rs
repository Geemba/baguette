use crate::*;
use sprite::*;

pub struct SpritePass
{
    render_pipeline: wgpu::RenderPipeline,
    buffers: Vec<std::ptr::NonNull<crate::sprite::SpriteGpuBinding>>,
    index_buffer: wgpu::Buffer,

    camera_bind_group: &'static wgpu::BindGroup
}

impl Drop for SpritePass
{
    fn drop(&mut self)
    {
        for bind in self.buffers.iter_mut()
        {
            unsafe { bind.as_mut().id.take(); }
        }
    }
}

impl SpritePass
{
    pub fn new(backface_culling: bool, cam: Option<&'static Camera>) -> Self
    {
        let cam = cam.unwrap_or_else(Camera::main);

        let shader = create_shader_module(wgpu::ShaderModuleDescriptor 
        {
            label: Some("spritesheet shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(r"sprite.wgsl").into())
        });

        let pipeline_layout = create_pipeline_layout(wgpu::PipelineLayoutDescriptor
        {
            label: Some("2d spritesheet pipeline layout"),
            bind_group_layouts:
            &[
                &bindgroup_layout(),
                &cam.binding.layout
            ],
            push_constant_ranges: &[]
        });

        let render_pipeline = create_render_pipeline(wgpu::RenderPipelineDescriptor 
        {
            label: Some("2d spritesheet pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState
            {
                module: &shader,
                entry_point: "vertex",
                buffers:
                &[
                    // for every vertex we pass the x and y local position
                    wgpu::VertexBufferLayout 
                    {
                        array_stride: core::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array!
                        [
                            0 => Float32, 1 => Float32
                        ]
                    },
                    // and for each instance we pass the model position(Transform) and an index
                    // that we'll use to get the uvs of the vertices from the storage buffer
                    wgpu::VertexBufferLayout
                    {
                        array_stride: core::mem::size_of::<([[f32; 4]; 4], u32)>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array!
                        [
                            2 => Float32x4,
                            3 => Float32x4,
                            4 => Float32x4,
                            5 => Float32x4,
                            6 => Uint32
                        ]
                    }
                ]
            },
            fragment: Some(wgpu::FragmentState
            {
                module: &shader,
                entry_point: "fragment",
                targets: 
                &[
                    Some(wgpu::ColorTargetState
                    {
                        format: config().format,
                        write_mask: wgpu::ColorWrites::ALL,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING)
                    })
                ]
            }),
            primitive: wgpu::PrimitiveState
            {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: match backface_culling 
                {
                    true => Some(wgpu::Face::Back),
                    false => None
                },
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None
        });

        Self
        {
            render_pipeline,
            camera_bind_group: &cam.binding.bindgroup,
            buffers: vec![],
            index_buffer: create_buffer_init
            (
                wgpu::util::BufferInitDescriptor 
                {
                    label: Some("spritesheet index buffer"),
                    contents: bytemuck::cast_slice(&SPRITE_INDICES),
                    usage: wgpu::BufferUsages::INDEX
                }
            )
        }
    }

    /// adds a new sprite to render, if another costructed [Sprite] has all the same parameters and texture
    /// you should add a new [SpriteInstance] inside of that struct using the instance parameter
    pub fn add<T>(&mut self, loader: SpriteLoader<T>) -> Sprite
        where
            T: Into<std::ffi::OsString> + AsRef<std::path::Path>
    {
        let mut sprite = SpriteBinding::from_loader(loader);

        self.buffers.push((sprite.binding.as_mut()).into());

        Sprite
        {
            spritebuffer: (&mut self.buffers).into(),
            // we use box so that we don't break references when moving values
            sprite,
        }
    }
}

impl RenderPass for SpritePass
{
    fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError>
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
        {
            label: Some("2d spritesheet render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment
            {
                view,
                resolve_target: None,
                ops: wgpu::Operations
                {
                    load: wgpu::LoadOp::Clear(wgpu::Color
                    {
                        r: 0.2,
                        g: 0.5,
                        b: 0.5,
                        a: 1.
                    }),
                    store: true
                }
            })],
            depth_stencil_attachment: None
        });
        
        render_pass.set_pipeline(&self.render_pipeline);
        
        for binding in self.buffers.iter()
        {
            let binding = unsafe { binding.as_ref() };

            render_pass.set_bind_group(0, &binding.bindgroup, &[]);
            render_pass.set_bind_group(1, self.camera_bind_group, &[]);
    
            render_pass.set_vertex_buffer(0, binding.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, binding.instance_buffer.0.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..SPRITE_INDICES.len() as u32, 0, 0..binding.instance_buffer.1);
        }

        Ok(())
    }

    fn add_pass() -> Passes where Self: Sized
    {
        Passes::SpriteSheet(Self::new(false, None))
    }
}

pub enum SpriteLoader<'a, Path>
    where
        Path: Into<std::ffi::OsString>,
{
    Sprite
    {
        /// directory of the source
        path: Path,
        /// describes how the sprite will be filtered,
        /// 
        /// [wgpu::FilterMode::Nearest] results in a pixelated effect
        /// while [wgpu::FilterMode::Linear] makes textures smooth but blurry
        filtermode: wgpu::FilterMode,
        instances: Vec<Transform>,
        /// describes how many pixels of this sprite
        /// represent one unit in the scene
        pxunit: f32
    },
    SpriteSheet
    {
        path: Path,
        /// describes how the sprite will be filtered,
        /// 
        /// [wgpu::FilterMode::Nearest] results in a pixelated effect
        /// while [wgpu::FilterMode::Linear] makes textures smooth but blurry
        filtermode: wgpu::FilterMode,
        layout: SpriteLayout,
        instances: Vec<(Transform, SheetTiles<'a>)>,
        /// describes how many pixels of this sprite
        /// represent one unit in the scene
        pxunit: f32
    }
}

#[derive(Clone)]
pub enum SheetTiles<'a>
{
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    Set(&'a [u32]),
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    RowColumn(&'a[(u32,u32)]),
    /// specify the tile indices using a [`std::ops::Range`] (`start..end`)
    /// 
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    Range(std::ops::Range<u32>),
    /// specify the tile indices using a [`std::ops::RangeInclusive`] (`start..=end`)
    /// 
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    RangeIn(std::ops::RangeInclusive<u32>)
}

impl SheetTiles<'_>
{
    pub(crate) fn into_indices(self, layout: SpriteLayout) -> Option<Tiles>
    {
        match self
        {
            SheetTiles::Set(val) => val.into_indices(layout),
            SheetTiles::RowColumn(val) => val.into_indices(layout),
            SheetTiles::Range(val) => val.into_indices(layout),
            SheetTiles::RangeIn(val) => val.into_indices(layout),
        }
    }
}

