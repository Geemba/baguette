use input::WindowEvent;

use crate::*;

/// a dynamically dispatched fsm that has been initialized
pub type InitDynFsm = dynamic::Fsm<dynamic::ActiveState>;

pub struct Application
{
    pub input: &'static mut input::Input,
    /// the application's renderer tasked with drawing to the screen
    pub renderer: rendering::Renderer,
    /// is the window focused
    pub focused: bool,
}

impl Application
{
    pub fn new(window: rendering::Window) -> Self
    {
        Self
        {
            input: input::Input::init(),
            renderer: rendering::Renderer::new(window),
            focused: true,
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
}

impl Application
{
    pub fn check_input(&mut self, event: &WindowEvent)
    {
        if !self.renderer.ui.handle_input(&self.renderer.window, event).consumed
        {
            self.input.check(event);
        }

        self.renderer.ui.begin_frame(&self.renderer.window)
    }
}