pub mod scene;
pub use baguette_proc::*;
pub use gameloop::*;

pub struct Application
{
    pub fsm : StateMachine,
    pub input : &'static mut input::Input,
    pub focused : bool,
}

pub enum GameEvent { FixedUpdate }

impl Application
{
    pub(crate) fn new
    (
        eventloop : &'static input::winit::event_loop::EventLoop<GameEvent>,
        fsm : StateMachine
    ) -> Self
    {
        let proxy = eventloop.create_proxy();

        //std::thread::spawn(move ||
        //{
        //    loop
        //    {
        //        spin_sleep::native_sleep(app.gameloop.fixed_step);
        //        proxy.send_event(GameEvent::FixedUpdate).ok();
        //    }          
        //});

        Self
        {
            fsm,
            input: input::Input::init(),
            focused: true
        }
    }

    /// invoke the update callback to all listeners,
    /// late update will be called in the same function just after update
    #[inline]
    pub(crate) fn update(&mut self)
    {
        self.fsm.update();
    }
    
    /// invoke the fixed update callback to all listeners
    #[inline]
    pub(crate) fn fixed_update(&mut self)
    {
        //self.gameloop.fixed_tick()
    }
}