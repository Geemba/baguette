use input::WindowEvent;

use crate::*;

/// a dynamically dispatched fsm that has been initialized
pub type InitDynFsm = dynamic::Fsm<dynamic::ActiveState>;

/// a statically dispatched fsm that has been initialized
pub type InitStaticFsm<T> = r#static::Fsm<r#static::ActiveState<T>,T>;

pub struct Application
{
    pub input: &'static mut input::Input,
    pub ui: ui::Context,
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
            focused: true,
            ui: ui::Context::default(),
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

impl Application
{
    pub fn check_input(&mut self, event: &WindowEvent)
    {
        self.input.check(event);

        let window = self.window();

        let scale_factor = window.scale_factor();
        let logical_size = window.inner_size().to_logical(scale_factor);

        self.ui.set_input(ui::egui::RawInput
        {
            screen_rect: Some(ui::egui::Rect::from_min_size
            (
                Default::default(), ui::egui::Vec2::new(logical_size.width, logical_size.height)
            )),
            max_texture_side: Some(self.renderer.max_texture_dimension() as usize),
            time: None,
            predicted_dt: 1./60.,
            ..Default::default()
        })
    }
}