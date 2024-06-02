use input::
{
    winit::
    {
        application::ApplicationHandler, event_loop::ActiveEventLoop, window::WindowAttributes
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
        Self
        {
            data: AppData::new(w_attributes),
            fsm
        }
    }
}

impl ApplicationHandler for AppHandler
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop)
    {
        //event_loop.create_window(self.window_attributes.clone());
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
    pub fn screen_size<T>(&self) -> input::winit::dpi::PhysicalSize<T>
        where T: input::winit::dpi::Pixel
    {
        self.renderer.screen_size::<T>()
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
}