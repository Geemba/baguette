//! the baguette engine aims to be a basic but performant engine for indie games,
//! providing all the necessary tools to develop a simple 2d game

pub use app;
pub use input;
pub use audio;
pub use rendering;
pub use math;

use app::*;
use input::winit::*;

pub type WindowTheme = window::Theme;

/// a dynamically dispatched fsm that is still unactive
pub(crate) type UninitDynFsm = FsmData<UnactiveState>;

#[must_use]
pub struct AppBuilder<T>
{
    /// keeps track of how to create the application window
    w_attributes: window::WindowAttributes,
    /// whether the app window will be focused or not
    focus: bool,
    fsm: T
}

/// same function as [new] but more fresh
pub fn fresh() -> AppBuilder<UninitDynFsm>
{
    self::new()
}

/// runs a new [AppBuilder] that store all your preferred options,
/// it defaults to `dynamic dispatch` to store the states
pub fn new() -> AppBuilder<UninitDynFsm>
{
    AppBuilder
    {
        w_attributes: window::Window::default_attributes(),
        fsm: Default::default(),
        focus: true
    }
}

pub type Transition<St> = (fn(&mut App, &St) -> bool, StateId);

impl AppBuilder<UninitDynFsm>
{
    /// adds a `state` to the `fsm` with the provided transitions 
    /// # examples
    /// ```
    /// fn example()
    /// {
    ///     baguette_core::new()
    ///     .set_theme(baguette_core::WindowTheme::Dark)
    ///     .add_state::<Test>(transitions!
    ///     [
    ///         |_| false => Test
    ///     ])
    /// ...
    /// ```
    /// # panics
    /// 
    /// panics if you attempt to add the same state twice
    pub fn add_state<St: dynamic::State + 'static>(mut self, transitions: fn() -> Vec<Transition<St>>) -> Self
    {
        let transitions = unsafe 
        {
            core::mem::transmute::
            <fn() -> Vec<(fn(&mut App, &St) -> bool, StateId)>,
             fn() -> Vec<(fn(&mut App, &DefaultDispatcher) -> bool, StateId)>
            >(transitions)
        };

        self.fsm.add_state::<St>(transitions);
        self
    }

    /// adds a non exiting loop to execute, use [add_state]
    /// to specify a transition condition to another existing state
    pub fn add_loop<St: dynamic::State + 'static>(mut self) -> Self
    {
        self.fsm.add_state::<St>(Vec::new);
        self
    }
    
    /// run the event loop
    pub fn run(self)
    {
        let eventloop = event_loop::EventLoop::new().unwrap();
        let _ = eventloop.run_app(&mut AppHandler::new(self.w_attributes, Fsm::Unactive(self.fsm)));
    }
}

impl<T> AppBuilder<T>
{
    pub fn set_title(mut self, title: impl Into<String>) -> Self
    {
        self.w_attributes = self.w_attributes.with_title(title);
        self
    }
    
    pub fn set_focus(mut self, value: bool) -> Self
    {
        self.w_attributes = self.w_attributes.with_active(value);
        self.focus = value;
        self
    }

    pub fn set_fullscreen(mut self) -> Self
    {
        self.w_attributes = self.w_attributes.with_fullscreen(Some(window::Fullscreen::Borderless(None)));
        self
    }

    pub fn set_maximized(mut self) -> Self
    {
        self.w_attributes = self.w_attributes.with_maximized(true);
        self
    }

    pub fn set_resizable(mut self, value: bool) -> Self
    {
        self.w_attributes = self.w_attributes.with_resizable(value);
        self
    }

    /// sets the window `icon` from a `byte slice`
    /// 
    /// # example
    /// 
    /// ```
    ///    baguette::new()
    ///    .with_title("example")    
    ///    .with_window_icon(include_bytes!("sprite.png"))
    /// 
    /// ```
    pub fn set_window_icon(mut self, bytes: &[u8]) -> Self
    {
        let (width, height);
        let bytes = match image::load_from_memory(bytes)
        {
            Ok(image) =>
            {
                (width, height) = image::GenericImageView::dimensions(&image);
                image.into_bytes()
            },
            Err(err) => panic!("{err}"),
        };

        self.w_attributes = self.w_attributes.with_window_icon(Some
        (
            match window::Icon::from_rgba(bytes, width, height)
            {
                Ok(icon) => icon,
                Err(err) => panic!("{err}"),
            }
        ));
        
        self
    }

    pub fn set_theme(mut self, theme: window::Theme) -> Self
    {
        self.w_attributes = self.w_attributes.with_theme(Some(theme));
        self
    }
}
