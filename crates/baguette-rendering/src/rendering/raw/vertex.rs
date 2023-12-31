#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex
{
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[allow(clippy::nonstandard_macro_braces)]
pub fn vertices_from_size_indexed(x: f32, y: f32) -> [Vertex; 4]
{
    [
        Vertex{ position:[-x, y, 0.0], tex_coords: [0., 0.] },
        Vertex{ position:[-x, -y, 0.0], tex_coords: [0., 1.] },
        Vertex{ position:[x, -y, 0.0], tex_coords: [1., 1.] },
        Vertex{ position:[x, y, 0.0], tex_coords: [1., 0.] },
    ]
}

#[allow(clippy::nonstandard_macro_braces)]
pub fn vertices_from_size(x: f32, y: f32) -> [Vertex; 6]
{
    [
        Vertex{ position:[-x, y, 0.0], tex_coords: [0., 0.] },
        Vertex{ position:[-x, -y, 0.0], tex_coords: [0., 1.] },
        Vertex{ position:[x, -y, 0.0], tex_coords: [1., 1.] },

        Vertex{ position:[x, -y, 0.0], tex_coords: [1., 1.] },
        Vertex{ position:[-x, y, 0.0], tex_coords: [0., 0.] },
        Vertex{ position:[x, y, 0.0], tex_coords: [1., 0.] },
    ]
}
/// creates an index array usable as index buffer to render a quad
pub fn indices_quad() -> Vec<u16>
{
    vec![0, 1 ,2 , 2, 3 ,0]
}

/// return how to read the vertex data to the gpu
pub const fn vertex_layout_desc<'a>() -> wgpu::VertexBufferLayout<'a> 
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
