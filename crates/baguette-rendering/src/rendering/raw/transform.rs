use crate::*;

pub struct TransformDescriptor
{
    /// world position of the transform, default is 0,0,0
    pub position : math::Vec3,
    /// world rotation of the transform, default is 0,0,0
    pub rotation : math::Vec3,
    /// world scale of the transform, default is 1,1,1
    pub scale : math::Vec3,
}

impl Default for TransformDescriptor
{
    fn default() -> Self 
    {
        Self
        {
            scale: math::Vec3::ONE,
            position: math::Vec3::ZERO,
            rotation: math::Vec3::ZERO,        
        }
    }
}

/// used to return transform as plain old data
pub type TransformRaw = [[f32;4];4];

pub struct Transform
{
    pub translation: math::Vec3,
    pub orientation: math::Quat,
    pub scale: math::Vec3,
}

impl Default for Transform
{
    fn default() -> Self 
    {
        let position = math::Vec3::ZERO;
        let rotation = math::Quat::IDENTITY;
        let scale = math::Vec3::ONE;

        Self
        {
            translation: position,
            orientation: rotation,
            scale
        }
    }
}

impl Transform
{
    pub fn new(desc : TransformDescriptor) -> Self 
    {
        let position = desc.position;
        let rotation = math::Quat::from_euler(math::EulerRot::XYZ, desc.rotation.x, desc.rotation.y, desc.rotation.z);
        let scale = math::Vec3::ONE;

        println!("{}", rotation);

        Self
        {
            translation: position,
            orientation: rotation,
            scale
        }
    }

    pub fn from_pos_rot_scale(position: math::Vec3, rotation: math::Quat, scale: math::Vec3) -> Self 
    {
        Self { translation: position, orientation: rotation, scale}  
    }

    pub fn set_scale(&mut self, scale: math::Vec3)
    {
        self.scale = scale;
    }

    pub fn to_raw(&self) -> TransformRaw
    {
        (math::Mat4::from_scale_rotation_translation(self.scale, self.orientation, self.translation)).to_cols_array_2d()
    }

}

pub fn to_raw(scale : math::Vec3, rotation : math::Quat ,position : math::Vec3,) -> TransformRaw
{
    (math::Mat4::from_scale_rotation_translation(scale, rotation, position)).to_cols_array_2d()  
}

pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> 
{
    wgpu::VertexBufferLayout 
    {
        array_stride: core::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
        // We need to switch from using a step mode of Vertex to Instance
        // This means that our shaders will only change to use the next
        // instance when the shader starts processing a new instance
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: 
        &[
            wgpu::VertexAttribute 
            {
                offset: 0,
                // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
            // for each vec4. We'll have to reassemble the mat4 in
            // the shader.
            wgpu::VertexAttribute 
            {
                offset: core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute 
            {
                offset: core::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute 
            {
                offset: core::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    }
}