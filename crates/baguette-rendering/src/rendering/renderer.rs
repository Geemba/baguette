use crate::*;
use input::winit::window::Window;

pub struct Renderer<'a>
{
    handle: &'a mut RendererHandler
}

impl<'a> From<&'a mut RendererHandler> for Renderer<'a> 
{
    fn from(handle: &'a mut RendererHandler) -> Self
    {
        Self
        {
            handle
        }
    }
}

impl Renderer<'_>
{
    pub fn ui(&self) -> Ui
    {
        (&self.handle.ui).into()
    }

    /// loads a sprite to be rendered,
    /// uses a builder type to describe how the sprite will be loaded
    pub fn load_sprite<T>(&mut self, sprite: SpriteLoader<T>) -> Sprite
        where
            T: Into<std::ffi::OsString> + AsRef<std::path::Path>
    {
        self.handle.get_or_insert_pass::<SpritePass>().add(sprite)
    }
}

/// this is handled by the engine
pub struct RendererHandler
{
    pub window: Window,
    pub ui: ui::UiHandle,

    output: FrameOutput,

    adapter: wgpu::Adapter,
    passes: Option<RenderPasses>
}

// integration specific
impl RendererHandler
{
    pub fn resize(&mut self, (width,height): (u32,u32))
    {
        config().width = width;
        config().height = height;

        let (physical_width, physical_height) = 
        (
            width as f32, height as f32
        );
        
        self.ui.update_screen_size(width, height);
        self.output.update_texture(device(), width, height);

        Camera::resize_all(physical_width / physical_height);

        self.update_surface()
    }

    /// returns a mutable reference to the pass of type `T` of this [`Renderer`], either by creating it if it's empty
    /// or by returning the existing one.
    fn get_or_insert_pass<T: RenderPass + 'static>(&mut self) -> &mut T
    {
        let passes = self.passes.get_or_insert_with(RenderPasses::new);

        // this is a bunch of boilerplate for type conversion 
        let pass = match (0..passes.renderpasses.len()).find
        (
            |&i| match &mut passes.renderpasses[i]
            {
                Passes::SpriteSheet(p) => p as &mut dyn std::any::Any,
            }.is::<T>()
        )
        {
            Some(i) => &mut passes.renderpasses[i],
            None =>
            {
                passes.renderpasses.push(<T>::add_pass());
                passes.renderpasses.last_mut().unwrap()
            }
        };
        (
            match pass
            {
                Passes::SpriteSheet(p) => p as &mut dyn std::any::Any,
            }         
        )
        // all type checking has been done before reaching this point so its safe to assume this is Some
        .downcast_mut().unwrap()
        
    }

    /// returns the render of this [`Renderer`].
    ///
    /// # Errors
    ///
    /// this function will return an error if the surface is not able to be retrieved.
    pub fn render
    (
        &mut self,
        window_target: &input::winit::event_loop::EventLoopWindowTarget<()>
    ) -> Result<(), wgpu::SurfaceError>
    {
        Camera::update_all();

        let output = surface().get_current_texture()?;
        let frame_output = &output.texture.create_view(&Default::default());

        let mut encoder = create_command_encoder("render encoder");

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
            {
                label: Some("renderer pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment
                {
                    view: &self.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations
                    {
                        load: wgpu::LoadOp::Clear(wgpu::Color
                        {
                            r: 0.13,
                            g: 0.31,
                            b: 0.85,
                            a: 1.0
                        }),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            });

            if let Some(passes) = &self.passes 
            {
                for render_pass in passes.iter()
                {
                    render_pass.draw(&mut pass)?
                }
            
                self.ui.render(&mut pass, &self.window, window_target);
            }
        }

        self.output.copy_to(&mut encoder, frame_output);
        
        queue().submit([encoder.finish()]);
        output.present();
        
        Ok(())
    }

    /// simply renders a solid color
    ///
    /// # Errors
    ///
    /// this function will return an error if the surface is not able to be retrieved.
    pub fn render_plain_color(&self, r:f64,g:f64,b:f64) -> Result<(), wgpu::SurfaceError>
    {
        Camera::update_all();

        let output = surface().get_current_texture()?;
        let frame_output = &output.texture.create_view(&Default::default());

        let mut encoder = create_command_encoder("render encoder");
        
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor
        {
            label: Some("renderer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment
            {
                view: frame_output,
                resolve_target: None,
                ops: wgpu::Operations
                {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1. }),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None
        });
        
        queue().submit([encoder.finish()]);
        output.present();
        
        Ok(())
    }

    /// can be used to release resources after rendering
    pub fn post_render(&mut self)
    {
        if self.passes.is_some() 
        {    
            self.ui.free_textures()
        }        
    }

    pub fn suspend(&mut self)
    {
        Screen::destroy();
    }

    /// required to be called for any change to [wgpu::Device] to be effective.
    /// 
    /// will update the surface with new config values
    ///
    fn update_surface(&mut self)
    {
        surface().configure(device(), config());
    }

    /// list all limits that were requested of this device.
    /// if any of these limits are exceeded, functions may panic.
    pub fn limits(&self) -> wgpu::Limits
    {
        device().limits()
    }

    pub fn begin_egui_frame(&mut self)
    {
        self.ui.begin_egui_frame(&self.window)
    }
}

/// initialization
impl RendererHandler
{
    /// Creates a new [`Renderer`].
    ///
    /// # Panics
    ///
    /// panics if an appropriate adapter or device is not avaiable.
    #[must_use]
    pub fn new(window: Window) -> Self
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
 
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions
        {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None
        })).expect("bruh failed to find an appropriate adapter");

        let (device, queue) = pollster::block_on
        (
            adapter.request_device
            (
                &wgpu::DeviceDescriptor
                {
                    label: Some("renderer device"),
                    features: adapter.features(),
                    limits: adapter.limits(),
                }, 
                None
            )
        ).expect("bruh failed to retrieve a device");

            // width and height of the rendered area in pixels
            let (width,height) = window.inner_size().into();

            // scalefactor of the screen we are rendering inside
            let scale = window.scale_factor() as f32;

        let output = FrameOutput::new(&device,width,height);
        
        static_render_data::StaticData::init(instance, device, queue);

        // until we dont remove the static data the order we itialize matters
        let ui = UiHandle::new(width,height,scale);

        Self { adapter, passes: None, window, ui, output }
    }

    /// this is where the window actually starts getting rendered.
    ///
    /// # Panics
    ///
    /// panics if the surface is not capable of being created.
    pub fn resume(&mut self)
    {    
        let surface = unsafe { instance().create_surface(&self.window) }
            .expect("failed to create window");
        
        let surface_caps = surface.get_capabilities(&self.adapter);

        //preferably srgb format
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration
        {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width: 1,
            height: 1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![]
        };

        ////

        Screen::init(surface, config);
   
        self.update_surface()
    }
}

/// handles how to present the final texture
struct FrameOutput
{
    view: wgpu::TextureView,
    bindgroup: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}
impl FrameOutput
{
    fn new(device: &wgpu::Device, width: u32,height: u32) -> Self 
    {
        let module = &device.create_shader_module
        (
            wgpu::ShaderModuleDescriptor
            {
                label: None,
                source: wgpu::ShaderSource::Wgsl
                (
                    include_str!("shaders/tex_to_tex_copy.wgsl").into()
                )
            }
        );

        let bindgroup_layout = &device.create_bind_group_layout
        (
            &wgpu::BindGroupLayoutDescriptor
            {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture 
                    {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },wgpu::BindGroupLayoutEntry
                {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None
                }]
            }
        );

        let view = device.create_texture
        (
            &wgpu::TextureDescriptor
            {
                label: Some("output texture"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }
        ).create_view(&Default::default());

        let sampler = device.create_sampler
        (
            &wgpu::SamplerDescriptor 
            {
                label: Some("output sampler"),
                ..Default::default()
            }
        );

        let bindgroup = device.create_bind_group
        (
            &wgpu::BindGroupDescriptor
            {
                label: Some("frame output bindgroup"),
                layout: bindgroup_layout,
                entries: &[wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view)
                },wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }]
            }
        );

        Self
        {
            bindgroup,
            pipeline: device.create_render_pipeline
            (
                &wgpu::RenderPipelineDescriptor
                {
                    label: Some("output render pipeline"),
                    layout: Some(&device.create_pipeline_layout
                    (
                        &wgpu::PipelineLayoutDescriptor
                        {
                            label: Some("output pipeline descriptor"),
                            bind_group_layouts: &[bindgroup_layout],
                            push_constant_ranges: &[]
                        }
                    )),
                    vertex: wgpu::VertexState
                    {
                        module, entry_point: "vertex", buffers: &[vertex_layout_desc()]
                    },
                    primitive: wgpu::PrimitiveState
                    {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Cw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some
                    (
                        wgpu::FragmentState 
                        {
                            module,
                            entry_point: "fragment",
                            targets: &[Some(wgpu::ColorTargetState
                            {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL
                            })]
                        }
                    ),
                    multiview: None
                }
            ),
            vertex_buffer: wgpu::util::DeviceExt::create_buffer_init
            (
                device,
                &wgpu::util::BufferInitDescriptor
                {
                    label: None,
                    contents: bytemuck::cast_slice(&vertices_from_size(1., 1.)),
                    usage: wgpu::BufferUsages::VERTEX
                }
            ),
            view,
            sampler
        }
    }

    /// update the texture to match the width and height arguments
    fn update_texture(&mut self, device: &wgpu::Device, width: u32,height: u32)
    {
        self.view = device.create_texture
        (
            &wgpu::TextureDescriptor
            {
                label: Some("output texture"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }
        ).create_view(&Default::default());

        let bindgroup_layout = &device.create_bind_group_layout
        (
            &wgpu::BindGroupLayoutDescriptor
            {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture 
                    {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },wgpu::BindGroupLayoutEntry
                {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None
                }]
            }
        );

        self.bindgroup = device.create_bind_group
        (
            &wgpu::BindGroupDescriptor
            {
                label: Some("frame output bindgroup"),
                layout: bindgroup_layout,
                entries: &[wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view)
                },wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler)
                }]
            }
        );  
    }

    /// copy output to this texture
    pub fn copy_to(&self, encoder: &mut wgpu::CommandEncoder, dest: &wgpu::TextureView)
    {
        let mut pass = encoder.begin_render_pass
        (
            &wgpu::RenderPassDescriptor
            {
                label: Some("copy texture renderpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment
                {
                    view: dest,
                    resolve_target: None,
                    ops: wgpu::Operations
                    {
                        load: Default::default(),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            }
        );

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bindgroup, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        pass.draw(0..6, 0..1)
    }
}

pub(super) mod static_render_data
{
    pub(super) struct StaticData
    {
        pub instance: wgpu::Instance,
        pub device: wgpu::Device,
        pub queue: wgpu::Queue,
        pub screen: Screen
    }

    pub struct Screen { surface: Option<wgpu::Surface>, config: wgpu::SurfaceConfiguration }

    impl Screen
    {
        /// initializes the screen
        ///
        /// # Panics
        ///
        /// panics if the static data is not initialized yet.
        pub fn init(surface : wgpu::Surface, config : wgpu::SurfaceConfiguration)
        {
            let screen = unsafe { &mut STATIC_DATA.get_mut().unwrap().screen };

            screen.surface = Some(surface);
            screen.config = config  
        }

        /// # Panics
        ///
        /// panics if the static data is not initialized.
        pub fn destroy() { unsafe { STATIC_DATA.get_mut().unwrap().screen.surface.take(); } }
    }

    impl StaticData
    {
        pub(super) fn init(instance: wgpu::Instance, device: wgpu::Device, queue: wgpu::Queue) 
        {
            assert!
            (
                // returns error if init was already called
                unsafe { &STATIC_DATA }.set
                (
                    Self 
                    {
                        instance,
                        device,
                        queue,
                        screen: Screen 
                        {
                            surface: None,
                            config: wgpu::SurfaceConfiguration
                            {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                width: 1,
                                height: 1,
                                present_mode: wgpu::PresentMode::Fifo,
                                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                                view_formats: vec![]
                            }
                        }
                    }
                ).is_ok(),
                "static rendering data must be initialized only once"
            );      
        }
    }

    static mut STATIC_DATA : once_cell::sync::OnceCell<StaticData> = once_cell::sync::OnceCell::new();

    #[inline]
    /// gets a reference to the instance of wgpu
    /// 
    /// will mostly be useful to create a surface
    /// # Panics
    ///
    /// panics if the static data is not initialized yet.
    pub fn instance() -> &'static wgpu::Instance
    {
        unsafe { &STATIC_DATA.get().unwrap().instance }
    }
    
    #[inline]
    /// gets a reference to the device instance
    /// # Panics
    ///
    /// panics if the static data is not initialized yet.
    pub fn device() -> &'static wgpu::Device
    {
        unsafe { &STATIC_DATA.get().unwrap().device }
    }

    #[inline]
    /// gets a mutable reference to the surface configuration
    /// # Panics
    ///
    /// panics if the static data is not initialized yet.
    pub fn config() -> &'static mut wgpu::SurfaceConfiguration
    {
        unsafe
        {
            &mut STATIC_DATA.get_mut().unwrap().screen
                .config
        }
    }

    #[inline]
    /// gets a reference to the surface if it exists
    /// # Panics
    ///
    /// panics if the static data is not initialized yet.
    pub fn surface() -> &'static wgpu::Surface
    {
        unsafe 
        {
            STATIC_DATA.get().unwrap().screen
                .surface.as_ref().unwrap()
        }
    }

    #[inline]
    /// gets a reference to the queue instance
    /// 
    /// A Queue executes recorded `CommandBuffer` objects and provides convenience methods 
    /// for writing to buffers and textures
    /// # Panics
    ///
    /// panics if the static data is not initialized yet.
    pub fn queue() -> &'static wgpu::Queue
    {
        unsafe { &STATIC_DATA.get().unwrap().queue }
    }
}
