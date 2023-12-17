use crate::*;

/// a dynamically dispatched fsm that has been initialized
pub type InitDynFsm = dynamic::Fsm<dynamic::ActiveState>;

/// a statically dispatched fsm that has been initialized
pub type InitStaticFsm<T> = r#static::Fsm<r#static::ActiveState<T>,T>;

pub struct Application
{
    pub input: &'static mut input::Input,
    pub renderer: rendering::Renderer,
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
            focused: true
        }
    }
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