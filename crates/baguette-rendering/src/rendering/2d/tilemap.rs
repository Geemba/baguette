use wgpu::*;
use crate::{*, util::TBuffer};

/// describes how to load a texture, how many rows and columns it has
#[derive(serde::Serialize, serde::Deserialize)]
struct TextureLoadDescriptor
{
    pub path: std::path::PathBuf,
    pub rows: u32, pub columns: u32, 
}

/// the textures have been passed
/// and the builder is ready to construct a tilemap
pub struct FullyConstructed;

/// until we dont call [self::TilemapBuilder::add_texture]
/// this is a useless builder, we need at least one texture
/// before we build a [Tilemap]
pub struct PartiallyConstructed;

#[derive(serde::Serialize, serde::Deserialize)]
/// Build a [`Tilemap`] with the given data.
/// It needs `at least` one texture to be able to construct it
pub struct TilemapBuilder
{
    /// how many images will be loaded
    /// to use with this tilemap
    maps: Vec<TextureLoadDescriptor>,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,

    layers: indexmap::IndexMap<u8, Vec<Tile>>,

    filter: FilterMode,
    pxunit: f32
}

impl TilemapBuilder
{
    /// Creates a [`TilemapBuilder`] from a tuple composed of the `path`, `rows` and `columns`
    pub fn with_textures(textures: &[(std::path::PathBuf, u32, u32)]) -> Self
    {
        let maps = textures.iter().map(|(path, rows, columns)| TextureLoadDescriptor
        {
            path: path.clone(),
            rows: *rows,
            columns: *columns

        }).collect();

        TilemapBuilder
        {
            maps,

            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,

            layers: Default::default(),
            filter: FilterMode::Nearest,
            pxunit: 100.,
        }
    }

    pub fn add_layer(mut self, layer: u8, tiles: impl IntoIterator<Item = Tile>) -> Self
    {
        self.layers.insert_sorted(layer, tiles.into_iter().collect());

        self
    }

    pub fn add_texture(mut self, path: impl Into<std::path::PathBuf>, rows: u32, columns: u32) -> Self
    {
        self.maps.push(TextureLoadDescriptor
        {
            path: path.into(),
            rows,
            columns,
        });

        self
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>>
    {
        bincode::deserialize::<Self>(bytes)
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, Box<bincode::ErrorKind>>
    {
        use std::io::*;

        let mut file = std::fs::File::open(&path)?;
        let mut bytes = vec![];

        file.read_to_end(&mut bytes)?;

        Self::from_bytes(&bytes)
    }
}

impl<const LEN: usize> From<&'static[u8; LEN]> for TilemapBuilder
{
    fn from(bytes: &'static[u8; LEN]) -> Self
    {
        Self::from_bytes(bytes).expect("invalid byte sequence")
    }
}

impl From<&'static str> for TilemapBuilder
{
    fn from(path: &str) -> Self
    {
        Self::from_path(path).expect("invalid path")
    }
}

#[derive(Default)]
pub(crate) struct TilemapPass
{
    pub layers: FastIndexMap<u8, Vec<Tile>>,
    tranform: Mat4,
    binding: Option<TilemapBinding>
}

impl TilemapPass
{
    pub fn add
    (
        &mut self, ctx: &ContextHandleInner,
        TilemapBuilder { maps, layers, filter, pxunit, position, rotation, scale }:
        TilemapBuilder
    )
    {
        self.tranform = Mat4::from_scale_rotation_translation(scale, rotation, position);

        let (textures, tilemap_data): (Vec<Texture>, Vec<TilemapData>) = maps
            .into_iter()
            .map(|tex_desc|
            {
                let texture = self::create_texture_from_path(ctx, tex_desc.path);

                let rows = tex_desc.rows as f32;
                let columns = tex_desc.columns as f32;

                const WIDTH: f32 = 0.5;
                const HEIGHT: f32 = 0.5;

                let tilemap_data =
                [
                    rows, columns,
                    -WIDTH, HEIGHT,
                    -WIDTH, -HEIGHT,
                    WIDTH, -HEIGHT,
                    WIDTH, HEIGHT,
                    0., 0. // padding
                ];

                (
                    texture,
                    tilemap_data
                )
            })
            .unzip();

        for (layer, mut new_tiles) in layers.into_iter()
        {
            match self.layers.get_mut(&layer)
            {
                Some(tiles) => tiles.append(&mut new_tiles),
                None =>
                {
                    self.layers.insert(layer, new_tiles);
                }
            }
        }

        if self.layers.is_empty()
        {
            self.layers.insert(0, vec![Tile::default()]);
        }

        let instances = self.layers.values().flatten().copied().collect::<Vec<_>>();

        if let Some(ref mut binding) = self.binding
        {
            binding.update_texture_bindgroup(ctx);
            binding.update_instances(ctx, &instances)
        }
        else
        {
            self.binding = Some(TilemapBinding::new
            (
                ctx, &textures, filter, &tilemap_data, &self.tranform, &instances, pxunit
            ))
        };
    }

    pub(crate) fn draw<'a>
    (
        &'a self,
        _: &ContextHandleInner,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData,
    )
    {
        let binding = self.binding.as_ref().unwrap();
        pass.set_pipeline(&binding.pipeline);

        pass.set_bind_group(0, &camera.bindings.bindgroup, &[]);
        pass.set_bind_group(1, &binding.matrix_bindgroup, &[]);
        pass.set_bind_group(2, &binding.tex_bindgroup, &[]);

        pass.set_vertex_buffer(0, binding.vert_buffer.slice(..));
        pass.set_vertex_buffer(1, binding.instance_buffer.slice(..));

        pass.set_index_buffer(binding.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        pass.draw_indexed(0..SPRITE_INDICES_U16.len() as _, 0, 0..binding.num_instances);
    }

    pub fn resize(&mut self, ctx: &ContextHandleInner)
    {
        if let Some(binding) = &mut self.binding
        {
            binding.resize_layers_texture(ctx)
        }
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

    vert_buffer: TBuffer<Uv>,
    index_buffer: TBuffer<u16>,
    tilemap_data_buffer: TBuffer<TilemapData>,
    instance_buffer: TBuffer<Tile>,
    matrix_buffer: TBuffer<Matrix>,
    layers_texture: TextureView,

    num_instances: u32
}

impl TilemapBinding
{
    pub fn new
    (
        ctx: &ContextHandleInner,
        textures: &[Texture],
        filter_mode: FilterMode,
        tilemap_data: &[TilemapData],

        transform: &Mat4,
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

        let layers_texture = Self::create_layers_texture(ctx);

        let shader = ctx.create_shader_module(include_wgsl!("tilemap.wgsl"));

        let pipeline = Self::create_pipeline(ctx, &shader, textures.len());

        let matrix = transform.to_cols_array_2d();

        let matrix_buffer = ctx.create_buffer_init
        (
            Some("tilemap matrix uniform"), &[matrix],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let vert_uv =
        [
            [0., 0.],
            [0., 1.],
            [1., 1.],
            [1., 0.],
        ];

        let vert_buffer = ctx.create_buffer_init
        (
            Some("tilemap vertex buffer"),
            &vert_uv,
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

        let tilemap_data_buffer = ctx.create_buffer_init
        (
            Some("tilemap_data_buffer"),
            tilemap_data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST
        );

        let views = textures.iter()
            .map(|texture| texture.create_view(&Default::default()))
            .collect::<Vec<_>>();

        Self
        {
            tex_bindgroup: Self::create_tilemap_textures_bindgroup
            (
                ctx, &views, &sampler, &tilemap_data_buffer, &layers_texture
            ),
            matrix_bindgroup: Self::create_matrix_bindgroup
            (
                ctx, &matrix_buffer
            ), 
            views,
            sampler,

            shader,
            pipeline,
            
            vert_buffer,
            index_buffer,
            matrix_buffer,
            instance_buffer,
            tilemap_data_buffer,
            
            layers_texture,
            num_instances: instances.len() as _,
        }
    }

    pub fn resize_layers_texture
    (
        &mut self,
        ctx: &ContextHandleInner
    )
    {
        self.layers_texture = Self::create_layers_texture(ctx);

        self.update_texture_bindgroup(ctx)
        
    }

    /// updates the texture bindgroup
    fn update_texture_bindgroup
    (
        &mut self,
        ctx: &ContextHandleInner,
    )
    {
        // objects to update: pipeline and tex bindgroup,
        self.tex_bindgroup = Self::create_tilemap_textures_bindgroup
        (
            ctx, &self.views, &self.sampler, &self.tilemap_data_buffer, &self.layers_texture
        );

        self.pipeline = Self::create_pipeline(ctx, &self.shader, self.views.len())
    }

    fn update_instances(&mut self, ctx: &ContextHandleInner, instances: &[Tile])
    {
        self.num_instances = instances.len() as u32;
        ctx.write_entire_buffer
        (
            &self.instance_buffer, instances
        )
    }

    fn create_pipeline(ctx: &ContextHandleInner, module: &ShaderModule, texture_count: usize) -> RenderPipeline
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
                        array_stride: std::mem::size_of::<Uv>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32, 1 => Float32],
                    },
                    VertexBufferLayout
                    {
                        array_stride: std::mem::size_of::<Tile>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &vertex_attr_array!
                        [
                            2 => Float32x2,
                            3 => Uint32,
                            4 => Uint32,
                            5 => Uint32,
                            6 => Float32
                        ]
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

    fn create_layers_texture
    (
        ctx: &ContextHandleInner

    ) -> TextureView
    {
        let (width, height) = ctx.screen.size();

        ctx.create_texture
        (
            TextureDescriptor
            {
                label: Some("2d layers texture"),
                size: Extent3d { width, height, ..Default::default() },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            }
        ).create_view(&Default::default())
    }

    fn create_tilemap_textures_bindgroup
    (
        ctx: &ContextHandleInner,
        views: &[TextureView],
        sampler: &Sampler,

        tilemap_data: &TBuffer<TilemapData>,
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
                    resource: BindingResource::Sampler(sampler)
                },
                BindGroupEntry
                {
                    binding: 2,
                    resource: BindingResource::TextureView(layer_texture)
                },
                BindGroupEntry
                {
                    binding: 3,
                    resource: tilemap_data.as_entire_binding()
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
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None
                },
                BindGroupLayoutEntry
                {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer
                    {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                }
            ]
        })
    }

    fn create_matrix_bindgroup
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

/// every vertex 
type Uv = [f32; 2];
type Matrix = [[f32; 4]; 4];

/// the first 2 floats contain `rows` and `columns`, the rest is padding
type TilemapData = [f32; 12];

#[repr(C)]
#[derive(Default, Clone, Copy)]
#[derive(Debug)]
#[derive(serde::Serialize,serde::Deserialize)]
pub struct Tile
{
    /// the position inside the tilemap
    pub pos: Vec2,
    /// the index of tile inside the tilemap texture
    pub row: u32,
    pub column: u32,
    /// the index of the tilemap texture,
    /// 
    /// the tilemap textures are indexed by the order you added them
    pub texture_idx: u32,
    pub layer: f32
}

unsafe impl bytemuck::NoUninit for Tile{}