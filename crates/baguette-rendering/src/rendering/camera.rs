use std::{cell::RefCell, sync::Arc};
use util::TBuffer;

use crate::*;

#[derive(Clone)]
/// a handle to the scenes camera
pub struct Camera
{
    pub(crate) data: Arc<RefCell<CameraData>>
}

impl Camera
{
    /// retrieve the camera from this renderer, 
    /// you can have only one camera for now
    pub fn get(renderer: &mut Renderer) -> Self
    {
        renderer.get_camera()
    }

    pub fn position(&self)-> Vec3
    {
        self.data.borrow().position()
    }

    pub fn set_position(&mut self, position: math::Vec3)
    {
        self.data.borrow_mut().set_position(position)
    }
}

/// a scene camera
pub(crate) struct CameraData
{
    pub projection: CameraProjection,
    pub bindings: CameraBinding
}

pub(crate) struct CameraBinding
{
    pub view_buffer: TBuffer<Mat4>,
    pub bindgroup: wgpu::BindGroup,
}

pub(crate) fn camera_bindgroup_layout(ctx: &ContextHandleInner) -> wgpu::BindGroupLayout
{
    ctx.create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor 
    {
        entries: &[wgpu::BindGroupLayoutEntry
        {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer
            {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bindgroup_layout"),
    })
}

#[must_use] fn get_binding_data(ctx: &ContextHandleInner) -> CameraBinding
{
    let buffer = ctx.create_buffer
    (
        Some("Camera Buffer"),
        core::mem::size_of::<Mat4>(),
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        false
    );

    let bind_group = ctx.create_bindgroup
    (
        Some("camera_bindgroup"),
        &camera_bindgroup_layout(ctx),
        &[
            wgpu::BindGroupEntry
            {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }
        ],
    );

    CameraBinding { view_buffer: buffer, bindgroup: bind_group }
}

impl CameraData
{
    pub(crate) fn new(ctx: &ContextHandleInner) -> Self
    {
        Self
        {
            projection: CameraProjection::new(&ctx.screen.config),       
            bindings: get_binding_data(ctx)
        }
    }

    pub(crate) fn resize(&mut self, aspect: f32)
    {
        self.projection.aspect = aspect;
        self.projection.rebuild_projection(aspect)        
    }

    pub(crate) fn update(&mut self, ctx: &parking_lot::RwLockReadGuard<'_, renderer::ContextHandleInner>)
    {
        // we rebuild the projection and pass it to the gpu as array
        let uniform = self.projection.screen_space_matrix();
    
        // and we queue a buffer write to update the actual matrix on the gpu
        ctx.write_entire_buffer(&self.bindings.view_buffer, &[uniform]);
    }

    #[inline]
    /// returns's the field of view (in radiants)
    pub const fn fov(&self) -> f32 { self.projection.fovy }

    #[inline]
    /// set's the field of view (in radiants)
    /// ```
    /// 
    /// 
    /// //example to zoom in
    /// self.set_fov(self.fov() - 0.1f32.to_radians())
    /// 
    /// ```
    pub fn set_fov(&mut self, mut fov: f32)
    {
        fov = fov.max(1f32.to_radians());

        self.projection.fovy = fov;
        self.projection.rebuild_projection(self.projection.aspect)
    }

    #[inline]
    pub fn set_projection_mode(&mut self, mode : ProjectionMode)
    {
        if self.projection.mode != mode
        {
            self.projection.mode = mode;
            self.projection.rebuild_projection(self.projection.aspect)
        }
    }

    #[inline]
    pub fn position(&self) -> math::Vec3
    {
        self.to_world_space(self.projection.translation)
    }

    #[inline]
    /// set's this camera's world position
    pub fn set_position(&mut self, position: math::Vec3)
    {
        self.projection.translation = self.to_view_space(position);
    }

    #[inline]
    pub const fn orientation(&self) -> math::Quat
    {
        self.projection.orientation
    }

    #[inline]
    pub fn rotate(&mut self, rotation: math::Quat)
    {
        let angles = rotation.to_euler(math::EulerRot::XYZ);

        self.projection.yaw -= angles.0;
        self.projection.pitch -= angles.1;
        self.projection.roll -= angles.2;

        self.projection.orientation = self.projection.recalculate_orientation()
    }

    /// aligns a camera space vector to global space
    #[inline]
    pub fn to_world_space(&self, pos: math::Vec3) -> math::Vec3
    {
        self.projection.orientation * pos
    }
    
    /// aligns a global space vector to camera space, this is not screen space
    #[inline]
    pub fn to_view_space(&self, pos: math::Vec3) -> math::Vec3
    {
        self.projection.orientation.conjugate() * pos
    }
}

#[derive(PartialEq, Eq)]
pub enum ProjectionMode
{
    Perspective,
    Orthographic
}

/// the projection of the camera
pub struct CameraProjection
{
    translation: math::Vec3,
    orientation: math::Quat,

    yaw: f32,
    pitch: f32,
    roll: f32,
    
    mode: ProjectionMode,
    projection: math::Mat4,

    aspect: f32,
    fovy: f32,
    near_clip: f32,
    far_clip: f32
}

impl CameraProjection
{
    fn new(config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let aspect = config.width as f32 / config.height as f32;
        let fovy = 45f32.to_radians();
        let near_clip = 0.01;
        let far_clip = 500.;

        Self
        {
            translation: math::Vec3::Z * 2.,
            orientation: math::Quat::IDENTITY,

            yaw: 0f32,
            pitch: 0f32,
            roll: 0f32,

            mode: ProjectionMode::Perspective,
            projection: math::Mat4::perspective_rh_gl(fovy, aspect, near_clip, far_clip),

            aspect,
            fovy,
            near_clip,
            far_clip
        }
    }

    #[inline]
    /// projection needs to be rebuild when any of these values change : `fovy`, `aspect`, `near_clip`, `far_clip`
    /// or projection mode is changed
    fn rebuild_projection(&mut self, aspect: f32)
    {
        self.projection = match self.mode
        {
            ProjectionMode::Perspective => Mat4::perspective_rh_gl(self.fovy, aspect, self.near_clip, self.far_clip),
            ProjectionMode::Orthographic => 
            {
                let top = self.fovy;
                let right = top * aspect;

                Mat4::orthographic_rh(-right, right, -top, top, self.near_clip, self.far_clip)
            }
        }
    }

    #[inline]
    /// converts the projection matrix to a buffer readable format
    fn screen_space_matrix(&self) -> Mat4
    {
        self.projection * self.view_matrix()
    }

    #[inline]
    fn view_matrix(&self) -> Mat4
    {
        Mat4::from_quat(self.orientation) * Mat4::from_translation(-self.translation)
    }
    
    #[inline]
    #[must_use]
    pub fn recalculate_orientation(&self) -> Quat
    {
        Quat::from_euler(EulerRot::XYZ, self.yaw, self.pitch, self.roll)
    }
}