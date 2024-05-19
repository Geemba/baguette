use crate::*;

/// this is the layout expected from the gpu for a [Transform]
pub(crate) type TransformRaw = [[f32; 4]; 4];

#[derive(Debug,PartialEq, Clone)]
pub struct Transform(Mat4);

impl Transform
{
    //pub const fn new(translation: math::Vec3, orientation: math::Quat, scale: math::Vec3) -> Self 
    //{
    //    Self { translation, orientation, scale }
    //}

    pub(crate) fn as_raw(&self) -> TransformRaw
    {
        self.0.to_cols_array_2d()
    }

    //pub fn set_scale(&mut self, scale: math::Vec3)
    //{
    //    self.scale = scale
    //}
}

/// describes the buffer layout of a [Transform]
pub const fn desc<'a>() -> wgpu::VertexBufferLayout<'a>
{
    wgpu::VertexBufferLayout
    {
        array_stride: core::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: 
        &[
            wgpu::VertexAttribute
            {
                offset: 0,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4
            },
            wgpu::VertexAttribute 
            {
                offset: core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4
            },
            wgpu::VertexAttribute 
            {
                offset: core::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4
            },
            wgpu::VertexAttribute 
            {
                offset: core::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4
            }
        ]
    }
}

impl From<Mat4> for Transform
{
    fn from(mat4: Mat4) -> Self
    {
        Self(mat4)
    }
}

//#[allow(clippy::nonstandard_macro_braces)]
//impl std::fmt::Display for Transform
//{
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
//    {
//        write!
//        (
//            f, "translation : {}, orientation : {}, scale {}",
//            self.translation, self.orientation, self.scale
//        )
//    }
//}
//
//impl Default for Transform
//{
//    fn default() -> Self 
//    {
//        let position = math::Vec3::ZERO;
//        let rotation = math::Quat::IDENTITY;
//        let scale = math::Vec3::ONE;
//
//        Self
//        {
//            translation: position,
//            orientation: rotation,
//            scale
//        }
//    }
//}

pub(crate) trait TransformCompute
{
    fn compute(&self) -> Mat4;
}