#[deprecated]
pub struct Mesh
{
    pub instances: Vec<super::Transform>,
    pub vertices: Vec<super::Vertex>,
    pub indices: Vec<u16>
}
#[allow(deprecated)]
impl Mesh
{
    pub fn create_vertex_buffer(&self, ctx: crate::ContextHandleData) -> wgpu::Buffer
    {
        ctx.create_buffer_init
        (
            wgpu::util::BufferInitDescriptor
            {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
        )
    }

    pub fn index_buffer(&self, ctx: crate::ContextHandleData) -> wgpu::Buffer
    {
        ctx.create_buffer_init
        (
            wgpu::util::BufferInitDescriptor 
            {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX
            }
        )
    }

    pub fn create_instance_buffer(&self, ctx: crate::ContextHandleData) -> wgpu::Buffer
    {
        let instances = self.instances.iter().map(|f| f.as_raw()).collect::<Vec<crate::TransformRaw>>();

        ctx.create_buffer_init
        (
            wgpu::util::BufferInitDescriptor
            {
                label: Some("instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
        )
    }
}
