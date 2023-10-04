#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex
{
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub fn vertices_from_size(size : baguette_math::Vec2) -> Vec<Vertex>
{
    vec!
    [
        Vertex{ position:[-size.x, size.y, 0.0], tex_coords: [0., 0.] },
        Vertex{ position:[-size.x, -size.y, 0.0], tex_coords: [0., 1.] },
        Vertex{ position:[size.x, -size.y, 0.0], tex_coords: [1., 1.] },
        Vertex{ position:[size.x, size.y, 0.0], tex_coords: [1., 0.] },
    ]
}

/// creates a 4 vertices index buffer from a texture
pub fn indices_from_texture() -> Vec<u16>
{
    vec!
    [
        0, 1 ,2 , 2, 3 ,0
    ]
}

/// return how to read the vertex data to the gpu
pub fn vertex_layout_desc<'a>() -> wgpu::VertexBufferLayout<'a> 
{
    wgpu::VertexBufferLayout 
    {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: 
        &[
            wgpu::VertexAttribute 
            {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute
            {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            }
        ]
    }
}
