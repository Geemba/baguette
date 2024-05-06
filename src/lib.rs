//! # Baguette engine
//! 
//! baguette aims to be a simple but performant engine for indie games,
//! providing all the necessary tools to develop a game

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


//enum FsmState
//{
//    Active(Fsm<ActiveState>),
//    Unactive(Fsm<UnactiveState>),
//    Dummy
//}
//
//impl FsmState
//{
//    fn build(&mut self, app: &mut App)
//    {
//        *self = match core::mem::replace(self, Self::Dummy)
//        {
//            FsmState::Unactive(unactive) => FsmState::Active(unactive.build(app)),
//            FsmState::Active(active) => FsmState::Active(active),
//            _ => unimplemented!(),
//        }
//    }
//
//    #[inline]
//    fn update(&mut self, mut app: App)
//    {
//        if let FsmState::Active(fsm) = self 
//        {
//            fsm.update(&mut app)
//        }
//    }
//}

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
            //let mut app = AppHandler::new
            //(
            //    eventloop.run_app(app)
            //    self.wbuilder
            //        .build(&eventloop)
            //        .expect("window creation has failed")
            //);

            // we should not start our states until event::resumed is invoked,
            // because our renderer is not initialized until that event,
        //    let mut fsm = FsmState::Unactive(self.fsm);

        //    eventloop.run
        //    (
        //        move |event: winit::event::Event<()>, target|
    
        //        match event
        //        {
        //            winit::event::Event::WindowEvent { event, .. } =>
        //            {
        //                app.check_input(&event);

        //                match event
        //                {
        //                    WindowEvent::RedrawRequested if app.focused => 
        //                    {
        //                        // begin gathering input before user update
        //                        app.renderer.begin_egui_frame();

        //                        fsm.update(app.to_user());

        //                        if let Some(err) = app.renderer.render(target).err()
        //                        {
        //                            match err
        //                            {
        //                                // Reconfigure the surface if it's lost or outdated
        //                                rendering::SurfaceError::Lost | rendering::SurfaceError::Outdated => println!("surface lost or outdated, reconnecting"),

        //                                // The system is out of memory, we should probably quit
        //                                rendering::SurfaceError::OutOfMemory => target.exit(),

        //                                rendering::SurfaceError::Timeout => println!("surface timeout")
        //                            }
        //                        }
        //                        
        //                        app.renderer.post_render();
        //                        app.input.flush_released_keys()
        //                    }
        //                    
        //                    WindowEvent::CloseRequested => target.exit(),
        //                    WindowEvent::Resized(new_size) if new_size.width > 0 && new_size.height > 0 =>
        //                    {
        //                        app.renderer.resize(new_size.into())
        //                    }
        //                    WindowEvent::Focused(value) => app.focused = value,
        //                    _ => ()
        //                }

        //                AppHandler::window(&app).request_redraw()
        //            }

        //            winit::event::Event::LoopExiting => (/* program exit */),

        //            winit::event::Event::Resumed =>
        //            {
        //                app.renderer.resume();
        //                fsm.build(&mut app.to_user())
        //            }
        //            winit::event::Event::Suspended =>
        //            {
        //                app.focused = false;
        //                app.renderer.suspend()
        //            }
        //            winit::event::Event::MemoryWarning => target.exit(),
        //            _ => ()
        //        }
        //    )
        //};

        //pollster::block_on(application).unwrap()
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
