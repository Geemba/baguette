#[path ="dispatch/dynamic.rs"]
pub mod dynamic;

pub mod application;
use std::any::{Any, TypeId};

pub use application::*;
pub use rendering::*;

pub use dynamic::*;

pub trait Dispatcher
{
    fn dispatch(&mut self, app: &mut App, event: &StateEvent);
}

impl Dispatcher for Box<dyn dynamic::AppState>
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
#[derive(Default)]
pub enum StateEvent
{
    #[default] Enter,
    Update,
    Exit(StateId)
}

pub(crate) struct Dummy;

#[derive(PartialEq, Debug, Eq, Hash, Clone, Copy)]
pub struct StateId(TypeId);

impl StateId
{
    pub(crate) fn of_value<T: AppState + ?Sized + 'static>(value: &T) -> Self
    {
        Self(Any::type_id(value))
    }

    pub fn of<T: AppState + Sized + 'static>() -> Self
    {
        Self(TypeId::of::<T>())
    }
}

impl Default for StateId
{
    fn default() -> Self
    {
        Self(TypeId::of::<Dummy>())
    }
}

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
    [$($lbracket:tt $first_closure:tt, $second_closure:tt $rbracket:tt $predicate:expr => $type:ident),*] =>
    {
       || vec![$(($lbracket $first_closure, $second_closure $rbracket $predicate, $type::id())),*]
    };
}

//pub trait OnBeforeScreenRedraw : Sync
//{
//    fn on_before_screen_redraw(&mut self);
//}
