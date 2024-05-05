
/// this is handled by the engine
pub struct AppHandler
{
    pub input: input::InputHandler,
    /// the application's renderer tasked with drawing to the screen
    pub renderer: rendering::RendererData,
    /// is the window focused
    pub focused: bool
}

impl AppHandler
{
    pub fn new(window: rendering::Window) -> Self
    {
        Self
        {
            input: Default::default(),
            renderer: rendering::RendererData::new(window),
            focused: true
        }
    }

    /// shortcut for &self.renderer.window
    #[inline]
    pub fn window(&self) -> &rendering::Window
    {
        &self.renderer.window  
    }

    #[inline]
    pub fn window_mut(&mut self) -> &mut rendering::Window
    {
        &mut self.renderer.window
    }

    pub fn check_input(&mut self, event: &input::WindowEvent)
    {
        // if egui doesn't consume this event, pass it to the rest of the program
        if !self.renderer.ui.handle_input(&self.renderer.window, event).consumed
        {
            self.input.check(event);
        }
    }

    /// return a wrapper type that doesn't contain the engine critical methods 
    pub fn to_user(&mut self) -> App
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
        self.ui().context().send_viewport_cmd(rendering::ui::ViewportCommand::Close)
    }

    pub fn screen_size<T>(&self) -> input::winit::dpi::PhysicalSize<T>
        where T: input::winit::dpi::Pixel
    {
        self.renderer.screen_size::<T>()
    }
}