use self::static_render_data::*;
use super::*;

pub struct Renderer
{
    adapter: wgpu::Adapter,
    passes : super::RenderPasses,
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
        if new_size.width <= 0 || new_size.height <= 0 { return }

        config().width = new_size.width;
        config().height = new_size.height;

        Camera::resize_all(new_size.width as f32 / new_size.height as f32);

        self.update_surface();
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError>
    {
        Camera::update_all();

        let output = surface().get_current_texture()?;
        let frame_output = &output.texture.create_view(&Default::default());

        let mut encoder = create_command_encoder("render encoder");

        for pass in self.passes.iter()
        {
            if let Err(err) = pass.draw(&mut encoder, &frame_output)
            {
                return Err(err)
            }
        };

        //for post in self.post_processes.iter()
        //{
        //    if let Err(err) = post.pass(&mut encoder, &frame_output, &self.post_processes.data)
        //    {
        //        return Err(err)
        //    }
        //}

        queue().submit([encoder.finish()]);
        output.present();
        
        Ok(())
    }

    /// adds tasks to execute when rendering
    pub fn add_render_pass(&mut self, pass : impl renderpasses::RenderPass + 'static)
    {
        self.passes.add_pass(pass)
    }

    //pub fn add_post_process_pass(&mut self, pass : impl PostProcessPass + 'static)
    //{
    //    self.post_processes.add_pass(pass)
    //}

    pub fn new(window : Window) -> Self
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor
        {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });
 
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions
        {
            power_preference: wgpu_types::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
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

        StaticData::init(instance, device, queue, window);

        let passes = RenderPasses::new();

        let mut renderer = Self { adapter, passes };

        crate::on_screen_resize().add_listener(&mut renderer);

        renderer
    }

    #[must_use]
    pub fn resume(&mut self)
    {
        println!("resume");

        let surface = unsafe { instance().create_surface(window()) }
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
            present_mode: wgpu_types::PresentMode::Fifo,
            alpha_mode: wgpu_types::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        ////

        Screen::init(surface, config);
   
        self.update_surface();  
    }

    pub fn suspend(&mut self)
    {
        Screen::destroy();
    }

    /// required to be called for any change to [Display] to be effective.
    /// 
    /// will update the surface with new config values
    /// 
    pub fn update_surface(&mut self)
    {
        surface().configure(&device(), config());
    }
}

pub mod static_render_data
{
    use super::*;

    pub(super) struct StaticData
    {
        pub instance: wgpu::Instance,
        pub device : wgpu::Device,
        pub window : Window,
        pub queue : wgpu::Queue,
        pub screen : Screen,
    }

    pub struct Screen { surface: Option<wgpu::Surface>, config: wgpu::SurfaceConfiguration }

    impl Screen
    {
        pub fn init(surface : wgpu::Surface, config : wgpu::SurfaceConfiguration)
        {
            let screen = unsafe { &mut STATIC_DATA.get_mut().unwrap_unchecked().screen };

            screen.surface = Some(surface);
            screen.config = config;          
        }

        pub fn destroy() { unsafe { STATIC_DATA.get_mut().unwrap_unchecked().screen.surface.take(); } }
    }

    impl StaticData
    {
        pub(super) fn init(instance: wgpu::Instance, device: wgpu::Device, queue: wgpu::Queue, window : Window) 
        {
            debug_assert!
            (
                // returns error if init was already called
                unsafe { &STATIC_DATA }.set
                (
                    Self 
                    {
                        instance,
                        device,
                        queue,
                        window,
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
    pub fn instance() -> &'static wgpu::Instance
    {
        unsafe { &STATIC_DATA.get_unchecked().instance }
    }
    
    #[inline]
    /// gets a reference to the device instance
    pub fn device() -> &'static wgpu::Device
    {
        unsafe { &STATIC_DATA.get_unchecked().device }
    }

    #[inline]
    /// gets a reference to the window
    pub fn window() -> &'static Window
    {
        unsafe { &STATIC_DATA.get_unchecked().window }
    }

    #[inline]
    /// gets a mutable reference to the surface configuration
    pub fn config() -> &'static mut wgpu::SurfaceConfiguration
    {
        unsafe
        {
            &mut STATIC_DATA.get_mut().unwrap_unchecked().screen
                .config
        }
    }

    #[inline]
    /// gets a reference to the surface if it exists
    pub fn surface() -> &'static wgpu::Surface
    {
        unsafe 
        { 
            STATIC_DATA.get_unchecked().screen
                .surface.as_ref().unwrap_unchecked()
        }
    }

    #[inline]
    /// gets a reference to the queue instance
    /// 
    /// A Queue executes recorded CommandBuffer objects and provides convenience methods 
    /// for writing to buffers and textures
    pub fn queue() -> &'static wgpu::Queue
    {
        unsafe { &STATIC_DATA.get_unchecked().queue }
    }
}