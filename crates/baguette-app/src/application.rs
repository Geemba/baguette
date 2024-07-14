use input::
{
    baguette_math::Vec2,
    winit::
    {
        application::ApplicationHandler,
        event_loop::ActiveEventLoop,
        window::WindowAttributes
    },
    WindowEvent
};

/// this is handled by the engine
pub(crate) struct AppData
{
    pub input: input::InputHandler,
    /// the application's renderer tasked with drawing to the screen
    pub renderer: rendering::RendererData,

    /// is the window focused
    pub focused: bool
}

pub struct AppHandler
{
    data: AppData,
    fsm: Fsm,
}

impl AppHandler
{
    pub fn new(w_attributes: WindowAttributes, fsm: Fsm) -> Self
    {
        setup_logger().unwrap();

        Self
        {
            data: AppData::new(w_attributes),
            fsm
        }
    }
}

/// initialize the logger
///
/// will return an error if it was already called
fn setup_logger() -> Result<(), log::SetLoggerError>
{
    use owo_colors::*;

    fern::Dispatch::new()
        .format
        (
            |out, message, record| out.finish
            (
                format_args!("[{}] {}: {}",

                match record.level()
                {
                    log::Level::Error => record.level().red().to_string(),
                    log::Level::Warn => record.level().yellow().to_string(),
                    log::Level::Info => record.level().green().to_string(),
                    log::Level::Debug => record.level().cyan().to_string(),
                    log::Level::Trace => record.level().blue().to_string(),
                },

                record.target().dimmed(),
                message)
            )
        )
        .level_for("wgpu_hal", log::LevelFilter::Error)
        .level_for("wgpu_core", log::LevelFilter::Error)
        .level_for("naga", log::LevelFilter::Warn)
        .level_for("wgpu", log::LevelFilter::Warn)
        .level(match cfg!(debug_assertions)
        {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Error
        })
        .chain(std::io::stdout())
        .apply()
}

impl ApplicationHandler for AppHandler
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop)
    {
        self.data.renderer.resume(event_loop);
        self.fsm.resume(&mut self.data.to_user_mut())
    }

    fn window_event
    (
        &mut self,
        target: &ActiveEventLoop,
        _window_id: input::winit::window::WindowId,
        event: input::WindowEvent,
    )
    {
        self.data.check_input(&event);

        match event
        {
            WindowEvent::RedrawRequested if self.data.focused =>
            {
                // begin gathering input before user update
                self.data.renderer.begin_egui_frame();
                
                self.fsm.update(&mut self.data.to_user_mut());

                if let Some(err) = self.data.renderer.render(target).err()
                {
                    match err
                    {
                        // Reconfigure the surface if it's lost or outdated
                        rendering::SurfaceError::Lost | rendering::SurfaceError::Outdated => println!("surface lost or outdated, reconnecting"),

                        // The system is out of memory, we should probably quit
                        rendering::SurfaceError::OutOfMemory => target.exit(),

                        rendering::SurfaceError::Timeout => println!("surface timeout")
                    }
                }
                
                self.data.renderer.post_render();
                self.data.input.flush_released_keys()
            }
            
            // do cleanup on exiting, not here,
            WindowEvent::CloseRequested => target.exit(),
            
            WindowEvent::Resized(new_size) if new_size.width > 0 && new_size.height > 0 =>
            {
                self.data.renderer.resize(new_size.into())
            }
            WindowEvent::Focused(value) => self.data.focused = value,
            _ => ()
        }

        self.data.window().request_redraw()
    }
    
    fn suspended(&mut self, _: &ActiveEventLoop)
    {
        self.data.focused = false;
        self.data.renderer.suspend()
    }
    
    fn exiting(&mut self, _event_loop: &ActiveEventLoop)
    {
        self.fsm.clear()
    }

    fn memory_warning(&mut self, target: &ActiveEventLoop)
    {
        target.exit()
    }
}

impl AppData
{
    pub fn new(w_attributes: WindowAttributes) -> Self
    {
        Self
        {
            input: Default::default(),
            renderer: rendering::RendererData::new(w_attributes),
            focused: true,
        }
    }

    /// shortcut for &self.renderer.window
    #[inline]
    pub fn window(&self) -> &rendering::Window
    {
        self.renderer.window.as_ref().unwrap()
    }

    //#[inline]
    //pub fn window_mut(&mut self) -> &mut rendering::Window
    //{
    //    self.renderer.window.as_mut().unwrap()
    //}

    pub fn check_input(&mut self, event: &input::WindowEvent)
    {
        // if egui doesn't consume this event, pass it to the rest of the program
        if !self.renderer.ui.handle_input(self.renderer.window.as_ref().unwrap(), event).consumed
        {
            self.input.check(event);
        }
    }

    /// return a wrapper that doesn't contain engine implementation methods 
    pub fn to_user_mut(&mut self) -> App
    {
        App
        {
            input: (&self.input).into(),
            renderer: (&mut self.renderer).into(),
        }
    }
}

pub struct App<'a>
{
    pub input: input::Input<'a>,
    /// the application's renderer tasked with drawing to the screen
    pub renderer: rendering::Renderer<'a>,
}

impl<'a> App<'a>
{
    pub fn ui(&self) -> rendering::ui::Ui
    {
        self.renderer.ui()
    }

    /// closes the program 
    pub fn close(&mut self)
    {
        // we just send a close command from egui,
        // this has no sense other than being faster to implement 
        // rather than creating more functions just to do the same thing
        self.ui().context().send_viewport_cmd(rendering::ui::egui::ViewportCommand::Close)
    }

    /// returns the screen size in the format you decide,
    /// ex:
    /// ```
    /// app.screen_size::<f32>()
    /// ```
    pub fn screen_size<T>(&self) -> (T,T)
        where T: input::winit::dpi::Pixel
    {
        self.renderer.screen_size::<T>()
    }

    /// returns the screen width in the format you decide,
    /// ex:
    /// ```
    /// let width = app.screen_width::<f32>()
    ///
    /// let width: f32 = self.screen_width();
    /// ```
    pub fn screen_width<T>(&self) -> T
        where T: input::winit::dpi::Pixel
    {
        self.renderer.screen_size::<T>().0
    }

    /// returns the screen heigth in the format you decide,
    /// ex:
    /// ```
    /// let heigth = app.screen_heigth::<f32>()
    ///
    /// let heigth: f32 = self.screen_heigth();
    /// ```
    pub fn screen_heigth<T>(&self) -> T
        where T: input::winit::dpi::Pixel
    {
        self.renderer.screen_size::<T>().1
    }
    
    pub fn get_key_down(&self, keycode: input::KeyCode) -> bool
    {
        self.input.get_key_down(keycode)
    }
    
    pub fn get_key_holding(&self, keycode: input::KeyCode) -> bool
    {
        self.input.get_key_holding(keycode)
    }
    
    pub fn get_key_up(&self, keycode: input::KeyCode) -> bool
    {
        self.input.get_key_up(keycode)
    }
    
    pub fn get_mouse_button_down(&self, click: input::MouseButton) -> bool
    {
        self.input.get_mouse_button_down(click)
    }
    
    pub fn get_mouse_button_holding(&self, click: input::MouseButton) -> bool
    {
        self.input.get_mouse_button_holding(click)
    }
    
    pub fn get_mouse_button_up(&self, click: input::MouseButton) -> bool
    {
        self.input.get_mouse_button_up(click)
    }
    
    pub fn horizontal_axis(&self) -> f32
    {
        self.input.horizontal_axis()
    }
    
    pub fn vertical_axis(&self) -> f32
    {
        self.input.vertical_axis()
    }
    
    pub fn input_axis(&self) -> Vec2
    {
        self.input.input_axis()
    }
}

pub enum Fsm
{
    Active(crate::FsmData<crate::ActiveState>),
    Unactive(crate::FsmData<crate::UnactiveState>),
    Dummy
}

impl Fsm
{
    fn resume(&mut self, app: &mut App)
    {
        *self = match core::mem::replace(self, Self::Dummy)
        {
            Fsm::Unactive(unactive) => Fsm::Active(unactive.build(app)),
            Fsm::Active(active) => Fsm::Active(active),
            _ => unimplemented!(),
        }
    }

    fn update(&mut self, app: &mut App)
    {
        if let Self::Active(fsm) = self
        {
            fsm.update(app)
        }
    }
    
    fn clear(&mut self)
    {
        *self = Self::Dummy
    }
}