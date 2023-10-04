//! fresh baguette
#[allow(unused_variables,dead_code)]
pub mod application;
use input::winit::*;
use application::{*};
use rendering::*;
pub use gameloop::*;

#[cfg(not(wasm_platform))]
pub async fn run(builder : AppBuilder)
{
    let eventloop = event_loop::EventLoopBuilder::<GameEvent>::with_user_event().build();

    let mut renderer = Renderer::new
    (
        builder.wbuilder
        .build(&eventloop)
        .expect("unable to create window")
    );

    renderer.add_render_pass(SpritePass::init(None, false));
    //renderer.add_post_process_pass(noise::NoisePostProcess::new(1., 0.));

    let mut app = Application::new
    (
        unsafe { core::mem::transmute(&eventloop) },
        builder.fsm.build(),
    );

    eventloop.run(move |event: Event<GameEvent>, _, control_flow: &mut event_loop::ControlFlow |
    
        match event
        {
            Event::WindowEvent { event, .. } =>
            {
                app.input.check(&event);
                
                if let WindowEvent::CloseRequested = event { control_flow.set_exit() }

                if let WindowEvent::Resized(new_size) = event { on_screen_resize().invoke(new_size) }

                if let WindowEvent::Focused(value) = event { app.focused = value }              
            }

            Event::MainEventsCleared if app.focused =>
            {
                app.update();
            
                match renderer.render()
                {
                    Ok(_) => (),
                    // Reconfigure the surface if it's lost or outdated
                    Err(rendering::SurfaceError::Lost | rendering::SurfaceError::Outdated) => println!("surface lost or outdated, reconnecting"),
                    
                    // The system is out of memory, we should probably quit
                    Err(rendering::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                    
                    Err(rendering::SurfaceError::Timeout) => println!("Surface timeout")
                }
                
                app.input.flush_released_keys()
            }

            Event::UserEvent(GameEvent::FixedUpdate) if app.focused => app.fixed_update(),   

            Event::LoopDestroyed => println!("on application quit"),

            Event::Resumed => renderer.resume(),

            Event::Suspended => renderer.suspend(),
   
            _ => ()
        }
    )
}

pub struct AppBuilder
{
    /// keeps track of how to create the application window
    wbuilder : window::WindowBuilder,
    /// whether the app window will be focused or not
    focus : bool,
    fsm : gameloop::UnactiveStateMachine

}

pub fn new() -> AppBuilder
{
    AppBuilder
    {
        wbuilder : window::WindowBuilder::new(),
        focus: true,
        fsm : gameloop::Fsm::new()
    }
}

pub type WindowTheme = window::Theme;
impl AppBuilder
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
    /// 
    /// panics if a state of the same type was already added 
    pub fn add_state<St: State + 'static>(mut self, transitions: fn() -> Vec<(fn(&St) -> bool, StateId)>) -> Self
    {
        let transitions =
        unsafe 
        {
            core::mem::transmute::
            <fn() -> Vec<(fn(&St) -> bool, StateId)>,
             fn() -> Vec<(fn(&dyn State) -> bool, StateId)>
            >(transitions)
        };

        self.fsm.add_state::<St>(transitions);

        self
    }
}

impl AppBuilder
{
    pub fn set_title<T: Into<String>>(mut self, title : T) -> Self
    {
        self.wbuilder = self.wbuilder.with_title(title);
        self
    }
    
    pub fn set_focus(mut self, value : bool) -> Self
    {
        self.wbuilder = self.wbuilder.with_active(value);
        self.focus = value;
        self
    }

    pub fn set_fullscreen(mut self) -> Self
    {
        self.wbuilder = self.wbuilder.with_fullscreen(Some(window::Fullscreen::Borderless(None)));
        self
    }

    pub fn set_maximized(mut self) -> Self
    {
        self.wbuilder = self.wbuilder.with_maximized(true);
        self
    }

    pub fn set_resizable(mut self, value : bool) -> Self
    {
        self.wbuilder = self.wbuilder.with_resizable(value);
        self
    }

    /// sets the window `icon` from a `byte slice`
    /// 
    /// # example
    /// 
    /// ```
    ///    baguette_framework::new()
    ///    .with_title("example")    
    ///    .with_window_icon(include_bytes!("sprite.png"))
    /// 
    /// ```
    pub fn set_window_icon(mut self, bytes : &[u8]) -> Self
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

        self.wbuilder = self.wbuilder.with_window_icon(Some
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
        self.wbuilder = self.wbuilder.with_theme(Some(theme));
        self
    }

    pub fn run(self)
    {
        pollster::block_on(crate::run(self));
    }
}