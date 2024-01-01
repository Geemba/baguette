//! link crate of all the `baguette` component crates

use app::*;
use input::{*,winit::*};

pub fn get_in_project_root(path : &str) -> String
{
    project_root() + path
}

/// returns the project root as string
/// # Panics
///
/// panics if the path is not found.
/// 
/// todo: way better to keep as os string
pub fn project_root() -> String
{
    project_root_path().unwrap().to_str().unwrap().to_owned()
}

pub fn project_root_path() -> std::io::Result<std::path::PathBuf> 
{
    for path in std::env::current_dir()?.as_path().ancestors()
    {
        if std::fs::read_dir(path)?.any
        (
            |p| p.unwrap().file_name() == *"Cargo.lock"
        )
        {
            return Ok(std::path::PathBuf::from(path))
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Ran out of places to find Cargo.toml"))
}

pub type WindowTheme = window::Theme;

/// a dynamically dispatched fsm that is still unactive
pub type UninitDynFsm = dynamic::Fsm<dynamic::UnactiveState>;

///// a statically dispatched fsm that is still unactive
//pub type UninitStaticFsm<T> = r#static::Fsm<r#static::UnactiveState<T>,T>;

/// a dynamically dispatched fsm that has been initialized
pub type InitDynFsm = dynamic::Fsm<dynamic::ActiveState>;

///// a statically dispatched fsm that has been initialized
//pub type InitStaticFsm<T> = r#static::Fsm<r#static::ActiveState<T>,T>;


enum FsmState
{
    Active(Fsm<ActiveState>),
    Unactive(Fsm<UnactiveState>),
    Dummy
}

impl FsmState
{
     fn build(&mut self, app: &mut Application)
    {
        *self = match core::mem::replace(self, Self::Dummy)
        {
            FsmState::Unactive(unactive) => FsmState::Active(unactive.build(app)),
            FsmState::Active(active) => FsmState::Active(active),
            _ => unimplemented!(),
        }
    }

    #[inline]
    fn update(&mut self, app: &mut Application)
    {
        if let FsmState::Active(fsm) = self 
        {
            fsm.update(app)
        }
    }
}

#[must_use]
pub struct AppBuilder<T>
{
    /// keeps track of how to create the application window
    wbuilder: window::WindowBuilder,
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
        wbuilder: window::WindowBuilder::new(),
        fsm: Default::default(),
        focus: true
    }
}

pub type Transition<St> = (fn(&St) -> bool, StateId);

impl AppBuilder<UninitDynFsm>
{
    ///// runs a new `app` using `custom dispatch` to store the states
    ///// 
    ///// # panics
    ///// 
    ///// panics if any state isn't subsequently added
    //pub fn with_dispatch<V : Dispatcher>(self) -> AppBuilder<UninitStaticFsm<V>>
    //{
    //    assert!
    //    (
    //        self.fsm.is_empty(),"set dispatch before adding your states 👮"
    //    );
    //    AppBuilder
    //    {
    //        wbuilder: self.wbuilder,
    //        focus: self.focus,
    //        // when a new static fsm is added it receives en empty state
    //        // as placeholder which is not able to be dispatched and therefore
    //        // throws a panic, so it's necessary to pass states to replace it
    //        fsm: UninitStaticFsm::new()
    //    }
    //}

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
    pub fn add_state<St: dynamic::State + 'static>(mut self, transitions: fn() -> Vec<Transition<St>>) -> Self
    {
        let transitions = unsafe 
        {
            core::mem::transmute::
            <fn() -> Vec<(fn(&St) -> bool, StateId)>,
             fn() -> Vec<(fn(&DefaultDispatcher) -> bool, StateId)>
            >(transitions)
        };

        self.fsm.add_state::<St>(transitions);
        self
    }

    /// adds a non exiting loop to execute, use [add_state]
    pub fn add_loop<St: dynamic::State + 'static>(mut self) -> Self
    {
        self.fsm.add_state::<St>(Vec::new);
        self
    }
    
    pub fn run(self)
    {
        let application = async
        {
            let eventloop = event_loop::EventLoopBuilder::default().build().unwrap();

            let mut app = Application::new
            (
                self.wbuilder
                    .build(&eventloop)
                    .expect("window creation has failed")
            );

            //we need to reach event::resumed before building the fsm
            let mut fsm = FsmState::Unactive(self.fsm);

            eventloop.run
            (
                move |event: Event<()>, target|
    
                match event
                {
                    Event::WindowEvent { event, .. } =>
                    {
                        app.check_input(&event);

                        match event
                        {
                            WindowEvent::RedrawRequested if app.focused => 
                            {
                                fsm.update(&mut app);

                                match app.renderer.render()
                                {
                                    Ok(_) => app.renderer.post_render(),
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(rendering::SurfaceError::Lost | rendering::SurfaceError::Outdated) => println!("surface lost or outdated, reconnecting"),

                                    // The system is out of memory, we should probably quit
                                    Err(rendering::SurfaceError::OutOfMemory) => target.exit(),

                                    Err(rendering::SurfaceError::Timeout) => println!("surface timeout")
                                }

                                app.input.flush_released_keys()
                            }
                            
                            WindowEvent::CloseRequested => target.exit(),
                            WindowEvent::Resized(new_size) => app.renderer.resize(new_size),
                            WindowEvent::Focused(value) => app.focused = value,
                            _ => ()
                        }

                        Application::window(&app).request_redraw()
                    }

                    Event::LoopExiting => (/* program exit */),

                    Event::Resumed =>
                    {
                        app.renderer.resume();
                        fsm.build(&mut app)
                    }

                    Event::Suspended =>
                    {
                        app.focused = false;
                        app.renderer.suspend()
                    },
                    Event::MemoryWarning => target.exit(),
                    _ => ()
                }
            )
        };

        pollster::block_on(application).unwrap()
    }
}

impl<T> AppBuilder<T>
{
    pub fn set_title(mut self, title: impl Into<String>) -> Self
    {
        self.wbuilder = self.wbuilder.with_title(title);
        self
    }
    
    pub fn set_focus(mut self, value: bool) -> Self
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

    pub fn set_resizable(mut self, value: bool) -> Self
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
}