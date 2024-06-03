use wgpu::*;
use crate::{*, util::TBuffer};

/// describes how to load a texture, how many rows and columns it has 
struct TextureLoadDescriptor
{
    pub path: std::path::PathBuf,
    pub rows: u16, pub columns: u16, 
}

/// the textures have been passed
/// and the builder is ready to construct a tilemap
pub struct FullyConstructed;

/// until we dont call [self::TilemapBuilder::add_texture]
/// this is a useless builder, we need at least one texture
/// before we build a [Tilemap]
pub struct PartiallyConstructed;

/// Build a [`Tilemap`] with the given data.
/// It needs `at least` one texture to be able to construct it
pub struct TilemapBuilder<T = PartiallyConstructed>
{
    /// how many images will be loaded
    /// to use with this tilemap
    maps: Vec<TextureLoadDescriptor>,
    tiles: Vec<Vec<Tile>>,
    filter: wgpu::FilterMode,
    pxunit: f32,
    phantom: std::marker::PhantomData<T>
}

impl TilemapBuilder
{
    /// Creates a partially constructed [`TilemapBuilder`].
    /// 
    /// It needs to call [`TilemapBuilder::add_texture`] before it can build a [Tilemap]
    pub fn new() -> TilemapBuilder<PartiallyConstructed>
    {
        TilemapBuilder
        {
            maps: vec![],
            tiles: vec![],
            pxunit: 100.,
            filter: FilterMode::Nearest,
            phantom: std::marker::PhantomData,
        }
    }    
}

impl<T> TilemapBuilder<T>
{ 
    pub fn set_layer(mut self, layer_index: u8, tiles: impl IntoIterator<Item = Tile>) -> Self
    {

        self
    }

    pub fn add_texture(mut self, path: impl Into<std::path::PathBuf>, rows: u16, columns: u16)
        -> TilemapBuilder<FullyConstructed>
    {
        self.maps.push(TextureLoadDescriptor
        {
            path: path.into(),
            rows,
            columns,
        });

        let Self
        {
            maps,
            tiles,
            filter,
            pxunit,
            ..
        } = self;

        TilemapBuilder::<FullyConstructed>
        {
            maps,
            tiles,
            filter,
            pxunit,
            phantom: std::marker::PhantomData,
        }
    }
}

impl Default for TilemapBuilder<PartiallyConstructed>
{
    fn default() -> Self { Self::new() }
}

#[derive(Default)]
pub(crate) struct TilemapPass
{
    tiles: Vec<Vec<Tile>>,
    binding: Option<TilemapBinding>
}

impl TilemapPass
{
    pub fn add
    (
        &mut self, ctx: crate::ContextHandle,
        TilemapBuilder { maps, tiles, filter, pxunit, .. }: TilemapBuilder<FullyConstructed>
    )
    {
        assert!
        (
            !maps.is_empty(),
            "no textures have been set to use in this tilemap, \n
            call TilemapBuilder::set_textures."
        );

        let ctx = &ctx.read().unwrap();

        let textures = maps.into_iter()
            .map(|TextureLoadDescriptor { path, .. }| self::create_texture_from_path(ctx, path))
            .collect::<Vec<_>>();

        if self.tiles.is_empty()
        {
            self.tiles.push(vec![Tile::default()])
        };
        
        let instances = self.tiles.iter().flatten().copied().collect::<Vec<_>>();

        self.binding = Some(TilemapBinding::new(ctx, &textures, filter, &instances, pxunit))
    }
}

impl crate::DrawPass for TilemapPass
{
    fn draw<'a>
    (
        &'a mut self,
        _: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData

    ) -> Result<(), SurfaceError>
    {
        let binding = self.binding.as_ref().unwrap();

        pass.set_pipeline(&binding.pipeline);

        pass.set_bind_group(0, &camera.bindings.bindgroup, &[]);
        pass.set_bind_group(1, &binding.matrix_bindgroup, &[]);
        pass.set_bind_group(2, &binding.tex_bindgroup, &[]);

        pass.set_vertex_buffer(0, binding.vert_buffer.slice(..));
        pass.set_vertex_buffer(1, binding.instance_buffer.slice(..));

        pass.set_index_buffer(binding.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        pass.draw_indexed(0..SPRITE_INDICES_U16.len() as _, 0, 0..self.tiles.len() as _);
        
        Ok(())
    }
}

struct TilemapBinding
{
    views: Vec<TextureView>,
    sampler: Sampler,
    
    shader: ShaderModule,
    pipeline: RenderPipeline,
    tex_bindgroup: BindGroup,
    matrix_bindgroup: BindGroup,

    vert_buffer: TBuffer<Vertex>,
    index_buffer: TBuffer<u16>,
    instance_buffer: TBuffer<Tile>,
    matrix_buffer: TBuffer<Matrix>,
    layers_texture: TextureView
}

impl TilemapBinding
{
    pub fn new
    (
        ctx: &ContextHandleInner,
        textures: &[Texture],
        filter_mode: FilterMode,

        instances: &[Tile],
        pxunit: f32
    ) -> Self
    {
        let (screen_width, screen_height) = ctx.screen.size();
        
        let sampler = ctx.create_sampler
        (
            SamplerDescriptor
            {
                address_mode_u: AddressMode::MirrorRepeat,
                address_mode_v: AddressMode::MirrorRepeat,
                address_mode_w: AddressMode::MirrorRepeat,
                mag_filter: filter_mode,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..Default::default()
            }
        );

        let layers_texture = ctx.create_texture(TextureDescriptor
        {
            label: None,
            size: Extent3d
            {
                width: screen_width.max(1),
                height: screen_height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Uint,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[],   
        })
        .create_view(&Default::default());

        let shader = ctx.create_shader_module(include_wgsl!("tilemap.wgsl"));

        let pipeline = Self::get_pipeline(ctx, &shader, textures.len());

        let matrix = Mat4::IDENTITY.to_cols_array_2d();

        let matrix_buffer = ctx.create_buffer_init
        (
            Some("tilemap matrix uniform"), &[matrix],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let Extent3d { width, height, .. } = textures[0].size();

        let scale = vec2(width as _, height as _) / pxunit;

        let vertices =
        [
            // [vertex.x, vertex.y, u, v]
            [-scale.x, scale.y , 0., 0.],
            [-scale.x, -scale.y, 0., 1.],
            [scale.x, -scale.y, 1., 1.],
            [scale.x, scale.y, 1., 0.],
        ];

        let vert_buffer = ctx.create_buffer_init
        (
            Some("tilemap vertex buffer"),
            &vertices,
            BufferUsages::VERTEX
        );

        let index_buffer = ctx.create_buffer_init
        (
            Some("tilemap index buffer"),
            &SPRITE_INDICES_U16,
            BufferUsages::INDEX
        );

        let instance_buffer = ctx.create_buffer_init
        (
            Some("tilemap vertex buffer"),
            instances,
            BufferUsages::VERTEX
        );

        let views = textures.iter()
            .map(|texture| texture.create_view(&Default::default()))
            .collect::<Vec<_>>();

        Self
        {
            tex_bindgroup: Self::get_tilemap_textures_bindgroup(ctx, &views, &sampler, &layers_texture),
            matrix_bindgroup: Self::get_matrix_bindgroup(ctx, &matrix_buffer), 
            views,
            sampler,

            shader,
            pipeline,
            vert_buffer,
            index_buffer,
            instance_buffer,
            matrix_buffer,
            layers_texture,
        }
    }

    fn update_bindings
    (
        &mut self,
        ctx: &ContextHandleInner,
    )
    {
        // objects to update: pipeline and tex bindgroup,
        self.tex_bindgroup = Self::get_tilemap_textures_bindgroup
        (
            ctx, &self.views, &self.sampler, &self.layers_texture
        );

        self.pipeline = Self::get_pipeline(ctx, &self.shader, self.views.len())
    }

    fn update_instances(&mut self, ctx: &ContextHandleInner, instances: &[Tile])
    {
        ctx.write_entire_buffer
        (
            &self.instance_buffer, instances
        )
    }

    fn get_pipeline(ctx: &ContextHandleInner, module: &ShaderModule, texture_count: usize) -> RenderPipeline
    {
        let pipe_layout = ctx.create_pipeline_layout
        (
            PipelineLayoutDescriptor
            {
                label: Some(&format!("tilemap pipeline layout, textures: {texture_count}")),
                bind_group_layouts: 
                &[
                    &crate::camera_bindgroup_layout(ctx),
                    &Self::get_tilemap_matrix_bindgroup_layout(ctx),
                    &Self::get_tilemap_textures_bindgroup_layout(ctx, texture_count as _)
                ],
                push_constant_ranges: &[]
            }
        );

        ctx.create_render_pipeline(RenderPipelineDescriptor
        {
            label: Some(&format!("tilemap render pipeline, textures: {texture_count}")),
            layout: Some(&pipe_layout),
            vertex: VertexState
            {
                module,
                entry_point: "vertex",
                buffers:
                &[
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<Tile>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &vertex_attr_array![2 => Float32x2, 3 => Uint32, 4 => Uint32]
                    }
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState
            {
                module,
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
        })
    }

    fn get_tilemap_textures_bindgroup
    (
        ctx: &ContextHandleInner,
        views: &[TextureView],
        sampler: &Sampler,

        layer_texture: &TextureView,
        
    ) -> BindGroup
    {
        let views = views.iter().collect::<Vec<_>>();

        ctx.create_bindgroup(BindGroupDescriptor
        {
            label: Some
            (
                &format!("tilemap bindgroup with {} textures", views.len())
            ),
            layout: &Self::get_tilemap_textures_bindgroup_layout(ctx, views.len() as _),
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
                    resource: BindingResource::Sampler(&sampler)
                },
                BindGroupEntry
                {
                    binding: 2,
                    resource: BindingResource::TextureView(layer_texture)
                }
            ]
        })
    }

    fn get_tilemap_matrix_bindgroup_layout(ctx: &ContextHandleInner) -> BindGroupLayout
    {
        ctx.create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor
        {
            label: Some("tilemap bind layout"),
            entries:
            &[
                BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer
                    {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
            ]
        })        
    }

    fn get_tilemap_textures_bindgroup_layout(ctx: &ContextHandleInner, count: u32) -> BindGroupLayout
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
                    count: None,
                },
                BindGroupLayoutEntry
                {
                    binding: 2,
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

    fn get_matrix_bindgroup
    (
        ctx: &ContextHandleInner,
        matrix_buffer: &Buffer,
    )
    -> wgpu::BindGroup
    {
        ctx.create_bindgroup(BindGroupDescriptor
        {
            label: Some("tilemap matrix bindgroup"),
            layout: &Self::get_tilemap_matrix_bindgroup_layout(ctx),
            entries:
            &[
                BindGroupEntry
                {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding()
                },
            ]
        })
    }
}

/// returns the texture and its pixel dimensions
fn create_texture_from_path
(
    ctx: &ContextHandleInner, source_path: impl AsRef<std::path::Path>,
)
-> Texture    
{
    use image::GenericImageView;

    let img = image::io::Reader::open(&source_path).unwrap()
        .decode().unwrap();

    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();

    let size = Extent3d
    {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let label = source_path.as_ref()
        .file_name().unwrap()
        .to_string_lossy();

    let texture = ctx.create_texture
    (
        TextureDescriptor 
        {
            label: Some(&label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        }
    );

    ctx.write_texture
    (
        ImageCopyTexture 
        {
            aspect: TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
        },
        &rgba,
        ImageDataLayout 
        {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        size
    );


    texture
}

///// copies a slice of [Tile] to a Vec of [RawTile]
//fn tiles_to_raw_tiles(instances: &[Tile]) -> Vec<RawTile>
//{
//    let instances: Vec<RawTile> = instances.iter().map
//    (
//        |Tile { pos, idx, texture_idx: bind_idx }| RawTile 
//        {
//            pos: pos.to_array(),
//            idx: *idx,
//            bind_idx: *bind_idx, 
//        }
//    )
//    .collect();
//
//    instances
//}

type Vertex = [f32; 4];
type Matrix = [[f32; 4]; 4];

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Tile
{
    /// the position inside the tilemap
    pub pos: Vec2,
    /// the index of tile inside the tilemap texture
    pub idx: u32,
    /// the index of the tilemap texture,
    /// 
    /// the tilemap textures are indexed by the order you added them
    pub texture_idx: u32,
}

unsafe impl bytemuck::NoUninit for Tile
{
    
}

//impl From<Tile> for RawTile
//{
//    fn from(Tile { pos, idx, texture_idx: bind_idx }: Tile) -> Self
//    {
//        Self
//        {
//            pos: pos.to_array(),
//            idx,
//            bind_idx,
//        }
//    }
//}