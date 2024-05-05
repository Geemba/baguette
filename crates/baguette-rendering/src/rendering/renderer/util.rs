use wgpu::{*, util::*};
use super::*;

impl ContextHandleData
{
    #[inline]
    pub fn create_shader_module(&self, desc: ShaderModuleDescriptor) -> ShaderModule
    {
        device(self).create_shader_module(desc)
    }
    #[inline]
    pub fn create_pipeline_layout(&self, desc: PipelineLayoutDescriptor) -> PipelineLayout
    {
        device(self).create_pipeline_layout(&desc)
    }
    #[inline]
    pub fn create_render_pipeline(&self, desc: RenderPipelineDescriptor) -> RenderPipeline
    {
        device(self).create_render_pipeline(&desc)
    }
    #[inline]
    pub fn create_bindgroup_layout(&self, desc: BindGroupLayoutDescriptor) -> BindGroupLayout
    {
        device(self).create_bind_group_layout(&desc)
    }
    #[inline]
    pub fn create_bindgroup(&self, desc: BindGroupDescriptor) -> wgpu::BindGroup
    {
        device(self).create_bind_group(&desc)
    }
    #[inline]
    pub fn create_buffer_init(&self, desc: BufferInitDescriptor) -> Buffer
    {
        device(self).create_buffer_init(&desc)
    }
    #[inline]
    pub fn create_buffer(&self, desc: BufferDescriptor) -> Buffer
    {
        device(self).create_buffer(&desc)
    }
        /// Schedule a data write into `buffer` with offset 0.
        ///
        /// This method is intended to have low performance costs.
        /// As such, the write is not immediately submitted, and instead enqueued
        /// internally to happen at the start of the next `submit()` call.
    #[inline]
    pub fn write_buffer<T: bytemuck::NoUninit>(&self, buffer : &Buffer, data : &[T])
    {
        queue(self).write_buffer(buffer, 0, bytemuck::cast_slice(data))
    }
    #[inline]
    pub fn create_sampler(&self, desc: SamplerDescriptor) -> Sampler
    {
        device(self).create_sampler(&desc)
    }
    #[inline]
    pub fn create_texture(&self, desc: TextureDescriptor) -> wgpu::Texture
    {
        device(self).create_texture(&desc)
    }

    #[inline]
    pub fn write_texture(&self, texture: ImageCopyTexture, data: &[u8], data_layout: ImageDataLayout, size: Extent3d)
    {
        queue(self).write_texture(texture, data, data_layout, size)
    }

    #[inline]
    pub fn create_texture_with_size(&self, width: u32, height: u32) -> wgpu::Texture
    {
        self.create_texture(TextureDescriptor
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
    pub fn create_command_encoder(&self, label: &str) -> wgpu::CommandEncoder
    {
        device(self).create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some(label) })
    }
}