use image::GenericImageView;
use wgpu::{*, util::DeviceExt};
use baguette_math::*;

pub struct Mesh
{
    pub instances : std::vec::Vec<super::TransformRaw>,
    pub vertices : Vec<super::Vertex>,
    pub indices : Vec<u16>   
}

impl Mesh
{
    pub fn create_vertex_buffer(&self) -> Buffer
    {
        super::device().create_buffer_init
        (
            &util::BufferInitDescriptor
            {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsages::VERTEX,
            }
        )
    }

    pub fn create_index_buffer(&self) -> Buffer
    {
        super::device().create_buffer_init
        (
            &util::BufferInitDescriptor 
            {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: BufferUsages::INDEX,
            }
        )
    }

    pub fn create_transform_buffer(&self,) -> wgpu::Buffer
    {
        wgpu::util::DeviceExt::create_buffer_init(super::device(),
        &wgpu::util::BufferInitDescriptor
        {
            label: Some("instances"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}

pub struct Sprite
{
    pub mesh : Mesh,
    pub texture : super::Texture,
    filter_mode : wgpu::FilterMode,
    id : u16,
}

impl PartialEq for Sprite
{
    fn eq(&self, other: &Self) -> bool
    {
        self.id == other.id &&
        self.filter_mode == other.filter_mode
    }
}

impl Sprite
{ 
    /// creates a sprite from file_path.
    /// 
    /// - filtermode will smooth out the individual pixel based on the passed mode,
    /// 
    ///   (linear is smooth but blurry, nearest gives a pixelated effect)
    /// 
    /// - max_size resizes images bigger than this to the given value
    pub fn from_path
    (
        file_path: &str, filter_mode : wgpu::FilterMode, max_size : Option<math::UVec2>,
    ) -> image::ImageResult<Sprite>
    {
        let path = std::path::Path::new(file_path);
        let file_name = path.file_name().unwrap().to_str().unwrap();

        match image::io::Reader::open(path).unwrap()
        .with_guessed_format().unwrap().decode()
        {
            Ok(img) => Ok(Self::from_image(img, filter_mode, file_name, max_size)),
            Err(err) => Err(err)
        }
    }

    pub fn from_image
    (
        mut img : image::DynamicImage,
        filter_mode : wgpu::FilterMode,
        label: &str,
        // will clamp the passed image to this size if is some
        max_size : Option<math::UVec2>,
    ) -> Sprite
    {

        if let Some(size) = max_size
        {
            img = img.resize(size.x,size.y,image::imageops::FilterType::Nearest);
        }

        let rgba = img.to_rgba8();

        let dimensions = img.dimensions();

        //println!("mesh size: {}, {}", dimensions.0, dimensions.1);

        let size = wgpu::Extent3d
        {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = super::static_render_data::device().create_texture
        (
            &wgpu::TextureDescriptor 
            {
                size,
                label : Some(label),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        super::queue().write_texture
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
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = super::static_render_data::device().create_sampler
        (
            &wgpu::SamplerDescriptor
            {
                address_mode_u: wgpu::AddressMode::MirrorRepeat,
                address_mode_v: wgpu::AddressMode::MirrorRepeat,
                address_mode_w: wgpu::AddressMode::MirrorRepeat,
                mag_filter: filter_mode,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        );

        let mesh_size = math::vec2(size.width as f32, size.height as f32).normalize();

        let mesh = super::Mesh
        {            
            vertices: super::vertex::vertices_from_size(mesh_size), 
            indices: vec!
            [
                0, 1 ,2 , 2, 3 ,0
            ],
            
            instances: vec!
            [
                super::Transform::default().to_raw(),
                //super::Transform::with_pos(device, glam::vec3(1., 0., 0.)).to_raw()
            ],
        };

        let mut id : u16 = 0;

        for byte in label.as_bytes()
        {
            id += *byte as u16;
        }

        Sprite
        {
            mesh,
            texture : super::Texture {texture, view, sampler},
            id,
            filter_mode,
        }
        
    }

    /// extends the sprite edges outside the texture bounds,
    /// negative values will cut the texture edges
    pub fn extend(&self, value : f32) 
    {
        for vertex in &self.mesh.vertices 
        {
            for ref mut axis in vertex.position
            {
                *axis *= value;
            }
        }
    }

}


