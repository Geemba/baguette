#[path ="dispatch/dynamic.rs"]
pub mod dynamic;

pub mod application;
pub use application::*;
pub use rendering::*;

pub use dynamic::*;

pub trait Dispatcher
{
    fn dispatch(&mut self, app: &mut App, event: &StateEvent);
}

pub type DefaultDispatcher = Box<dyn dynamic::State>;


impl Dispatcher for DefaultDispatcher
{
    fn dispatch(&mut self, app: &mut App, event: &StateEvent)
    {
        self.update(app, event)
    }
}

/// if you need a certain operation to execute when first entering
/// this [State] you can match this enum.
/// 
/// # Example
/// ```
/// 
///     fn update(&mut self, event: &StateEvent)
///     {
/// 
///         match event
///         {
///             /// this gets executed only when first entering
///             StateEvent::Enter => todo!(),
///             /// this keeps getting executed after enter has been invoked once
///             StateEvent::Update => todo!(),
///             /// this gets executed only when exiting
///             StateEvent::Exit(_) => todo!(),
///         }
/// 
///         if let StateEvent::Enter = event
///         {
///             /// this gets execute only when first entering
///         }
///         ...
///     }
/// ```
pub enum StateEvent
{
    Enter,
    Update,
    Exit(StateId)
}

impl Default for StateEvent
{
    /// the default state is enter
    fn default() -> Self { StateEvent::Enter }
}

#[derive(PartialEq,Debug, Eq, Hash, Clone, Copy)]
pub struct StateId(core::any::TypeId);

pub struct Empty;

#[macro_export]
/// changes state if the predicate returns `true`
/// ```
/// transitions!
/// [
///     |_| false => Test
/// ])
/// ```
macro_rules! transitions
{
    [$($lbracket:tt $closure:tt $rbracket:tt $predicate:expr => $type:ident),*] =>
    {
       || vec![$(($lbracket $closure $rbracket $predicate, $type::id())),*]
    };
}

pub trait OnBeforeScreenRedraw : Sync
{
    fn on_before_screen_redraw(&mut self);
}
