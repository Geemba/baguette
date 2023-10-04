use wgpu::{*, util::*};
use super::*;

#[inline]
pub fn create_shader_module(desc : ShaderModuleDescriptor) -> ShaderModule
{
    device().create_shader_module(desc)
}
#[inline]
pub fn create_pipeline_layout(desc : PipelineLayoutDescriptor) -> PipelineLayout
{
    device().create_pipeline_layout(&desc)
}
#[inline]
pub fn create_render_pipeline(desc : RenderPipelineDescriptor) -> RenderPipeline
{
    device().create_render_pipeline(&desc)
}
#[inline]
pub fn create_bindgroup_layout(desc : BindGroupLayoutDescriptor) -> BindGroupLayout
{
    device().create_bind_group_layout(&desc)
}
#[inline]
pub fn create_bindgroup(desc : BindGroupDescriptor) -> wgpu::BindGroup
{
    device().create_bind_group(&desc)
}
#[inline]
pub fn create_buffer_init(desc : BufferInitDescriptor) -> Buffer
{
    device().create_buffer_init(&desc)
}
#[inline]
pub fn create_buffer(desc : BufferDescriptor) -> Buffer
{
    device().create_buffer(&desc)
}
#[inline]
pub fn write_buffer<T : bytemuck::Pod>(buffer : &Buffer, data : T)
{
    queue().write_buffer(buffer, 0, bytemuck::cast_slice(&[data]))
}
#[inline]
pub fn create_sampler(desc : SamplerDescriptor) -> Sampler
{
    device().create_sampler(&desc)
}
#[inline]
pub fn create_texture(desc : TextureDescriptor) -> wgpu::Texture
{
    device().create_texture(&desc)
}

#[inline]
pub fn create_texture_with_size(width : u32, height : u32) -> wgpu::Texture
{
    create_texture(TextureDescriptor
    {
        label: None,
        size: wgpu::Extent3d
        {
            width: std::cmp::max(width, 1),
            height: std::cmp::max(height, 1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],   
    })
}

#[inline]
pub fn create_command_encoder(label : &str) -> wgpu::CommandEncoder
{
    device().create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some(&label) })
}
