use crate::*;

//static mut POST_PROCESS : Option<PostProcessPasses> = None;

pub type PostProcess = PostProcessPasses;

pub struct PostProcessPasses
{
    pub data : PostProcessData,
    renderpasses : Vec<Box<dyn PostProcessPass>>
}

impl std::ops::Index<usize> for PostProcessPasses
{
    type Output = Box<dyn self::PostProcessPass>;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.renderpasses[index]
    }
}

impl std::ops::IndexMut<usize> for PostProcessPasses
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output 
    {
        &mut self.renderpasses[index]
    }
}

impl PostProcessPasses
{
    //
    //pub fn init()
    //{ 
    //    POST_PROCESS.get_or_insert
    //    (
    //        Self 
    //    {
    //        renderpasses : vec![], data : PostProcessData::new()
    //    }); 
    //}

    pub fn len(&self) -> usize { self.renderpasses.len() }

    pub fn is_empty(&self) -> bool
    {
        self.len() > 0
    }

    pub fn add_pass<'a>(&mut self, pass : impl self::PostProcessPass + 'a + 'static)
    {
        self.renderpasses.push(Box::new(pass))
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Box<dyn self::PostProcessPass>>
    {
        self.renderpasses.iter_mut()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Box<dyn self::PostProcessPass>>
    {
        self.renderpasses.iter()
    }

    //pub fn post_processing() -> Option<&'static PostProcess>
    //{
    //    unsafe { POST_PROCESS }
    //}

    // returns the processing view if post process is active
    //pub fn processing_view() -> &'static wgpu::TextureView
    //{
    //    unsafe { &POST_PROCESS.as_ref().unwrap().data.processing }
    //}

    ///// returns the processed view if post process is active 
    //pub fn processed_view() -> &'static wgpu::TextureView
    //{
    //    unsafe { &POST_PROCESS.as_ref().unwrap().data.processed }
    //}

    ///// returns the sampler if post process is active 
    //pub fn sampler() -> &'static wgpu::Sampler
    //{
    //    unsafe { &POST_PROCESS.as_ref().unwrap().data.sampler }
    //}
}

pub trait PostProcessPass
{
    fn pass
    (
        &self, encoder : &mut wgpu::CommandEncoder,
        attachment_view : &wgpu::TextureView, post_data : &PostProcessData
    ) -> Result<(), wgpu::SurfaceError>;

    /// will be called when the post processor
    /// 
    /// resizes the view, requiring to recreate all associated bindings
    /// 
    /// to use the new view
    fn update_bindings(&mut self, attachment_view : &wgpu::TextureView, sampler : &wgpu::Sampler);

    //fn get_post_processor(&self) -> &PostProcess { unsafe{ post_process() } }
}

pub struct PostProcessData
{
    pub processing : wgpu::TextureView,
    pub processed : wgpu::TextureView,
    pub sampler : wgpu::Sampler,
    pub custom_resolution : Option<(u32, u32)>,
    pub vertex_buffer : wgpu::Buffer,
    pub index_buffer : wgpu::Buffer,
}

impl Default for PostProcessData
{
    fn default() -> Self 
    {
        Self::new()
    }
}

impl PostProcessData
{
    pub fn new() -> Self
    {
        //two triangles covering the entire screen
        let screen_vertices = 
        [
            super::Vertex { position: [-1., 1., 0.], tex_coords: [0., 0.] },
            super::Vertex { position: [-1., -1., 0.], tex_coords: [0., 1.] },
            super::Vertex { position: [1., -1., 0.], tex_coords: [1., 1.] },
            super::Vertex { position: [1., 1., 0.], tex_coords: [1., 0.] }
        ];
    
        let vertex_buffer = create_buffer_init(wgpu::util::BufferInitDescriptor
        {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&screen_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    
        let indices: [u16;6] =
        [
            0, 1, 2, 2, 3, 0
        ];
    
        let index_buffer = create_buffer_init(wgpu::util::BufferInitDescriptor
        {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let processing = Self::recreate_view(None);
        let processed = Self::recreate_view(None);
        let sampler = create_sampler(wgpu::SamplerDescriptor::default());

        Self 
        {
            vertex_buffer, index_buffer,
            processing, processed, sampler,
            custom_resolution: None,
        }
    }

    pub fn recreate_view(size : Option<(u32, u32)>) -> wgpu::TextureView
    {
        let size = size.map_or_else(|| (config().width, config().height), |size| size);

        create_texture
        (
            wgpu::TextureDescriptor
            {
                label: Some("frame output"),
                size: wgpu::Extent3d
                {
                    width: size.0,
                    height: size.1,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: config().format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[]
            }
        ).create_view(&Default::default())
    }

    pub fn resize(&mut self)
    {
        self.processing = Self::recreate_view(self.custom_resolution);
        self.processed = Self::recreate_view(self.custom_resolution);
    }
}