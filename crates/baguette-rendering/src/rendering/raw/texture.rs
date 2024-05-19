
use image::GenericImageView;

pub struct TextureData
{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub pxunit: f32,
}

impl TextureData 
{
    pub fn from_bytes
    (
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str
    ) -> Option<Self>
    {

        image::load_from_memory(bytes)
        .map_or_else
        (
            |_| None, |img| Self::from_image(device, queue, &img, Some(label))
        )       
    }

    pub fn from_image
    (
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>
    ) -> Option<Self>
    {                                                                                                                               
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d
        {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture
        (
            &wgpu::TextureDescriptor 
            {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture
        (
            wgpu::ImageCopyTexture 
            {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout 
            {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler
        (
            &wgpu::SamplerDescriptor
            {
                address_mode_u: wgpu::AddressMode::MirrorRepeat,
                address_mode_v: wgpu::AddressMode::MirrorRepeat,
                address_mode_w: wgpu::AddressMode::MirrorRepeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        );

        Some(Self { texture, view, sampler, pxunit: 0. })
    }

    pub fn size(&self) -> baguette_math::Vec2
    {
        let size = self.texture.size();
        baguette_math::Vec2::new(size.width as f32, size.height as f32) 
    }

}
