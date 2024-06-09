use wgpu::{*, util::*};
use super::*;
use bytemuck::NoUninit;

/// a [wgpu::Buffer] with type safety on top of it
pub struct TBuffer<T: bytemuck::NoUninit>
(
    wgpu::Buffer,
    std::marker::PhantomData<T>
);


impl<T: NoUninit> AsRef<wgpu::Buffer> for TBuffer<T>
{
    fn as_ref(&self) -> &wgpu::Buffer
    {
        &self.0
    }
}

impl<T: NoUninit> From<wgpu::Buffer> for TBuffer<T>
{
    fn from(value: wgpu::Buffer) -> Self
    {
        Self(value, std::marker::PhantomData)
    }
}

impl<T: NoUninit> std::ops::Deref for TBuffer<T>
{
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl ContextHandleInner
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
    pub fn create_buffer_init<T: NoUninit>
    (
        &self,
        label: Option<&str>,
        contents: &[T],
        usage: BufferUsages
    ) -> TBuffer<T>
    {
        let descr = BufferInitDescriptor
        {
            label,
            contents: bytemuck::cast_slice(contents),
            usage,
        };

        device(self).create_buffer_init(&descr).into()
    }
    #[inline]
    pub fn create_buffer<T: NoUninit>
    (
        &self,
        label: Option<&str>,
        size: usize,
        usage: BufferUsages,
        mapped_at_creation: bool,
    ) -> TBuffer<T>
    {
        let desc = BufferDescriptor
        {
            label,
            size: size as u64,
            usage,
            mapped_at_creation
        };

        device(self).create_buffer(&desc).into()
    }
    /// Rewrite the entire buffer with this data slice 
    ///
    /// This method is intended to have low performance costs.
    /// As such, the write is not immediately submitted, and instead enqueued
    /// internally to happen at the start of the next `submit()` call.
    #[inline]
    pub fn write_entire_buffer<T: NoUninit>(&self, buffer: &TBuffer<T>, data: &[T])
    {
        queue(self).write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }

    //#[inline]
    //pub fn write_buffer_with_offset<T: bytemuck::NoUninit>(&self, buffer : &GpuBuffer<T>,offset: u64, data : &[T])
    //{
    //    queue(self).write_buffer(buffer, offset, bytemuck::cast_slice(data))
    //}

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
    pub fn create_command_encoder(&self, label: &str) -> wgpu::CommandEncoder
    {
        device(self).create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some(label) })
    }
}