use crate::*;

pub struct Renderer
{
    pub window: Window,

    adapter: wgpu::Adapter,
    passes: Option<RenderPasses>
}

impl crate::CallbackListener<winit::dpi::PhysicalSize<u32>> for Renderer
{
    fn callback_listener(&mut self, new_size : winit::dpi::PhysicalSize<u32>)
    {
        self.resize(new_size)
    }
}

impl Renderer
{
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        config().width = u32::max(new_size.width, 1);
        config().height = u32::max(new_size.height, 1);

        Camera::resize_all(new_size.width as f32 / new_size.height as f32);

        self.update_surface_inner()
    }

    pub fn get_or_insert_pass<T: RenderPass + 'static>(&mut self) -> &mut T
    {
        let passes = self.passes.get_or_insert_with(RenderPasses::new);

        let pass = match (0..passes.renderpasses.len()).find
        (
            |&i| match passes.renderpasses[i]
            {
                Passes::SpriteSheet(ref mut p) => p as &mut dyn std::any::Any
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
                Passes::SpriteSheet(p) => p
            }
            as &mut dyn std::any::Any
        )
        // all type checking has been done before we reach this point so its safe to assume this is Some
        .downcast_mut().unwrap()
        
    }

    /// returns the render of this [`Renderer`].
    ///
    /// # Errors
    ///
    /// this function will return an error if the surface is not able to be retrieved.
    pub fn render(&self) -> Result<(), wgpu::SurfaceError>
    {
        Camera::update_all();

        let output = surface().get_current_texture()?;
        let frame_output = &output.texture.create_view(&Default::default());

        let mut encoder = create_command_encoder("render encoder");

        match &self.passes
        {
            Some(passes) => for pass in passes.iter()
            {
                pass.draw(&mut encoder, frame_output)?
            }   
            
            None => 
            {
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor
                {
                    label: Some("empty renderer pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment
                    {
                        view: frame_output,
                        resolve_target: None,
                        ops: wgpu::Operations
                        {
                            load: wgpu::LoadOp::Clear(wgpu::Color
                            {
                                r: 0.1,
                                g: 0.2,
                                b: 0.5,
                                a: 1.0
                            }),
                            store: true
                        }
                    })],
                    depth_stencil_attachment: None,
                });
            }
        }

        queue().submit([encoder.finish()]);
        output.present();
        
        Ok(())
    }

    pub fn suspend(&mut self)
    {
        Screen::destroy();
    }

    /// required to be called for any change to [wgpu::Device] to be effective.
    /// 
    /// will update the surface with new config values
    ///
    fn update_surface_inner(&mut self)
    {
        surface().configure(device(), config());
    }

    pub fn get_pass<T>(&mut self) -> Option<&mut T> where T: RenderPass + 'static
    {
        if let Some(ref mut passes) = self.passes 
        {
            return passes.iter_mut().find_map
            (
                |pass| match pass
                {
                    Passes::SpriteSheet(pass) => (pass as &mut dyn core::any::Any).downcast_mut::<T>()
                }
            )
        }
        None
    }
}

/// 2d specific
impl Renderer
{
    /// loads a sprite to be rendered
    pub fn load_sprite<T>(&mut self, sprite: SpriteLoader<T>) -> Sprite
        where
            T: Into<std::ffi::OsString> + AsRef<std::path::Path>
    {
        self.get_or_insert_pass::<SpritePass>().add(sprite)
    }
}

/// initialization
impl Renderer
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
        ).expect("bruh failed to create device");

        static_render_data::StaticData::init(instance, device, queue);

        let mut renderer = Self { adapter, passes: None, window };

        crate::on_screen_resize().add_listener(&mut renderer);

        renderer
    }

    /// Returns the resume of this [`Renderer`].
    ///
    /// # Panics
    ///
    /// panics if the window is not capable of being recreated.
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
   
        self.update_surface_inner()
    }
}

#[cfg(debug_assertions)]
impl Renderer
{
    pub fn config_size(&self) -> (u32,u32)
    {
        (config().width, config().height)
    }

    pub fn update_surface(&mut self)
    {
        self.update_surface_inner();      
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