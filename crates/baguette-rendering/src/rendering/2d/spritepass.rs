use crate::*;
use sprite::*;

pub struct SpritePass
{
    render_pipeline: wgpu::RenderPipeline,
    buffers: Vec<std::ptr::NonNull<crate::sprite::SpriteGpuBinding>>,
    index_buffer: wgpu::Buffer,
}

impl Drop for SpritePass
{
    fn drop(&mut self)
    {
        for binding in self.buffers.iter_mut()
        {
            // this notifies the sprites 
            // that the spritepass has been dropped,
            // it is expected that this id value exists when dropping the spritepass
            unsafe
            {
                binding.as_mut().id.take().expect
                (
                    "value should have been present in the moment of dropping the spritepass"
                )
            };
        }
    }
}

impl SpritePass
{
    pub fn new(backface_culling: bool, ctx: ContextHandle) -> Self
    {
        let ctx_read = ctx.read().unwrap();
        let shader = ctx_read.create_shader_module(wgpu::ShaderModuleDescriptor 
        {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!(r"sprite.wgsl").into())
        });

        let pipeline_layout = ctx_read.create_pipeline_layout(wgpu::PipelineLayoutDescriptor
        {
            label: Some("2d sprite pipeline layout"),
            bind_group_layouts:
            &[
                &bindgroup_layout(&ctx_read),
                &camera_bindgroup_layout(&ctx_read)
            ],
            push_constant_ranges: &[]
        });

        let render_pipeline = ctx_read.create_render_pipeline(wgpu::RenderPipelineDescriptor 
        {
            label: Some("2d sprite pipeline"),
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
                        array_stride: core::mem::size_of::<[f32; 2]>() as u64,
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
                        array_stride: core::mem::size_of::<([[f32; 4]; 4], u32)>() as u64,
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
                        format: ctx_read.screen.config.format,
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
            multisample: wgpu::MultisampleState::default(),
            depth_stencil: None,
            multiview: None
        });

        Self
        {
            render_pipeline,
            buffers: vec![],
            index_buffer: ctx_read.create_buffer_init
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
    pub fn add<T>(&mut self, ctx: crate::ContextHandle, loader: SpriteLoader<T>) -> Sprite
        where
            T: Into<std::ffi::OsString> + AsRef<std::path::Path>
    {
        let mut sprite = SpriteBinding::from_loader(ctx ,loader);

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
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, camera: &'a camera::CameraData) -> Result<(), wgpu::SurfaceError>
    {
        pass.set_pipeline(&self.render_pipeline);

        for binding in self.buffers.iter()
        {
            let binding = unsafe { binding.as_ref() };

            pass.set_bind_group(0, &binding.bindgroup, &[]);
            pass.set_bind_group(1, &camera.binding.bindgroup, &[]);
    
            pass.set_vertex_buffer(0, binding.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, binding.instance_buffer.0.slice(..));

            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..SPRITE_INDICES.len() as u32, 0, 0..binding.instance_buffer.1);
        }

        Ok(())
    }

    fn add_pass(ctx: ContextHandle) -> Passes where Self: Sized
    {
        Passes::SpriteSheet(Self::new(false, ctx))
    }
}

/// describes the type of sprite you want to create
pub enum SpriteLoader<'a, Path: Into<std::ffi::OsString>>
{
    /// sas
    SingleSprite
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

