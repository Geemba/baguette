use crate::*;

static mut CAMERAS: std::vec::Vec<Camera> = vec![];

/// a scene camera
pub struct Camera
{
    pub projection: CameraProjection,
    pub binding: GpuBinding
}

pub struct GpuBinding
{
    pub buffer: wgpu::Buffer,
    pub bindgroup: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout
}

#[must_use] fn get_binding_data() -> GpuBinding
{
    let buffer = create_buffer(wgpu::BufferDescriptor
    {
        label: Some("Camera Buffer"),
        size: core::mem::size_of::<[[f32; 4]; 4]>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false
    });

    let layout = create_bindgroup_layout(wgpu::BindGroupLayoutDescriptor 
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
    });

    let bind_group = create_bindgroup(wgpu::BindGroupDescriptor
    {
        layout: &layout,
        entries:
        &[
            wgpu::BindGroupEntry
            {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }
        ],
        label: Some("camera_bindgroup"),
    });

    GpuBinding { buffer, bindgroup: bind_group, layout }
}

impl Default for Camera
{
    fn default() -> Self
    {
        Self
        {
            projection: CameraProjection::default(),       
            binding: get_binding_data()
        }
    }
}

/// returns the main camera as mutable if there is one, or returns a new instance and sets it as main
#[inline]
#[must_use] pub fn main_mut() -> &'static mut Camera
{
    Camera::main_mut()
}

/// returns the main camera if there is one, or returns a new instance and sets it as main
#[inline]
#[must_use] pub fn main() -> &'static Camera
{
    Camera::main()
}

//static
impl Camera
{   
    /// returns all existing cameras in the scene
    pub fn all() -> &'static Vec<Self>
    {
        unsafe { &CAMERAS }
    }

    /// returns all existing cameras in the scene
    fn all_mut() -> &'static mut Vec<Self>
    {
        unsafe { &mut CAMERAS }
    }
    
    /// returns the main camera if there is one, otherwise returns a new instance and sets it as main
    pub fn main() -> &'static Self
    {
        Self::main_mut()
    }
    /// returns the main camera if there is one, otherwise returns a new instance and sets it as main
    pub fn main_mut() -> &'static mut Self
    {
        unsafe
        {
            match CAMERAS.get_mut(0) 
            {
                Some(cam) => cam,
                None => 
                {
                    CAMERAS.push(Camera::default());
                    &mut CAMERAS[0]
                }
            }   
        }
    }

    pub(crate) fn resize_all(aspect: f32)
    {
        for cam in Self::all_mut().iter_mut()
        {
            cam.projection.aspect = aspect;
            cam.projection.rebuild_projection(aspect)
        }
    }

    pub(crate) fn update_all()
    {
        for cam in Self::all_mut().iter_mut()
        {
            cam.update()
        }
    }
}

impl Camera
{
    pub(crate) fn update(&mut self)
    {
        // we rebuild the projection and pass it to the gpu as array
        let uniform = self.projection.screen_space_matrix().to_cols_array_2d();
    
        // and we queue a buffer write to update the actual matrix on the gpu
        crate::write_buffer(&self.binding.buffer, &uniform);
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

impl Default for CameraProjection
{
    fn default() -> Self 
    {
        let aspect = config().width as f32 / config().height as f32;
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
}

impl CameraProjection
{
    #[inline]
    /// projection needs to be rebuild when any of these values change : `fovy`, `aspect`, `near_clip`, `far_clip`
    /// or projection mode is changed
    fn rebuild_projection(&mut self, aspect: f32)
    {
        self.projection = match self.mode
        {
            ProjectionMode::Perspective => math::Mat4::perspective_rh_gl(self.fovy, aspect, self.near_clip, self.far_clip),
            ProjectionMode::Orthographic => 
            {
                let top = self.fovy;
                let right = top * aspect;

                math::Mat4::orthographic_rh(-right, right, -top, top, self.near_clip, self.far_clip)
            }
        }
    }

    #[inline]
    /// converts the projection matrix to a buffer readable format
    fn screen_space_matrix(&self) -> math::Mat4
    {
        self.projection * self.view_matrix()
    }

    #[inline]
    fn view_matrix(&self) -> math::Mat4
    {
        math::Mat4::from_quat(self.orientation) * math::Mat4::from_translation(-self.translation)
    }
    
    #[inline]
    #[must_use]
    pub fn recalculate_orientation(&self) -> math::Quat
    {
        math::Quat::from_euler(math::EulerRot::XYZ, self.yaw, self.pitch, self.roll)
    }
}