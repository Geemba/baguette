use std::{num::NonZeroU32, sync::RwLockReadGuard};

use baguette_math::{Mat4, Vec2};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Buffer, BufferUsages, ShaderStages, SurfaceError};

type ContextRead<'a> = RwLockReadGuard<'a, ContextHandleData>;

use crate::
{
    CameraData, ContextHandle, ContextHandleData, Passes, RenderPass, TextureData
};

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

impl RenderPass for TilemapPass
{
    fn add_pass(ctx: ContextHandle) -> Passes
    {
        Passes::Tilemap(TilemapPass::new(ctx))
    }

    fn draw<'a>
    (
        &'a mut self,
        ctx: &ContextRead,
        pass: &mut wgpu::RenderPass<'a>,
        camera: &'a CameraData

    ) -> Result<(), SurfaceError>
    {
        pass.set_bind_group(0, &camera.bindings.bindgroup, &[]);

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
    bindgroup: BindGroup,
    textures: Vec<TextureData>,
    mat_buffer: Buffer
}

impl TilemapBinding
{
    pub fn new(ctx: ContextHandle) -> Self
    {
        let ctx = ctx.read().unwrap();

        let textures = vec![];

        let matrix = Mat4::IDENTITY.to_cols_array_2d();

        let mat_buffer = ctx.create_buffer_init(wgpu::util::BufferInitDescriptor
        {
            label: Some("tilemap matrix uniform"),
            contents: bytemuck::cast_slice(&matrix),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self
        {
            bindgroup: Self::create_bindgroup(&ctx, &textures, &mat_buffer),
            textures,
            mat_buffer,
        }
    }

    fn create_bindgroup(ctx: &ContextRead, textures: &[TextureData], matrix_buffer: &Buffer) -> BindGroup
    {
        let views = textures.iter().map(|data| &data.view).collect::<Vec<_>>();
        let samplers = textures.iter().map(|data| &data.sampler).collect::<Vec<_>>();

        ctx.create_bindgroup(BindGroupDescriptor
        {
            label: Some("tilemap bindgroup"),
            layout: &Self::create_layout(ctx),
            entries: &
            [
                BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views)
                },
                BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(&samplers)
                },
                BindGroupEntry
                {
                    binding: 2,
                    resource: matrix_buffer.as_entire_binding()
                }
            ]
        })
    }

    fn create_layout(ctx: &ContextRead) -> BindGroupLayout
    {
        ctx.create_bindgroup_layout(
            wgpu::BindGroupLayoutDescriptor
            {
                label: Some("tilemap bind layout"),
                entries: &
                [
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
                        count: NonZeroU32::new(1),
                    }
                ]
            }
        )
    }
}