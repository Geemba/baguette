//! statically dispatched
use crate::*;

/// [Transition] is composed of a `predicate` function and the [StateId]
/// it should transition to if the predicate returns `true`
pub type Transition<T> = (fn(&T) -> bool, StateId);

/// a function pointer that returns the state concrete type
type StateReturnCallback<T> = fn() -> T;

/// a function pointer that returns a vec containing all the possible transitions
type TransitionsCallback<T> = fn() -> Vec<Transition<T>>;

pub trait State<T : Dispatcher>
{
    fn new() -> Self where Self : Sized;

    fn update(&mut self, event : &StateEvent);

    fn id() -> StateId
    where
        Self: core::any::Any + Sized
    {
        StateId(core::any::TypeId::of::<Self>())
    }

    fn into_dispatch() -> T;
}

/// an empty state to use as default
impl<T : Dispatcher> State<T> for Empty
{
    fn new() -> Self where Self : Sized { Self }
    fn update(&mut self, _ : &StateEvent) {}

    fn into_dispatch() -> T { panic!("attempted to build fsm with no state 👮") }
}

pub struct ActiveState<T : Dispatcher>
{
    id : StateId,
    state : T,
    event : StateEvent,
    transitions : Vec<Transition<T>>,
    to_unactive : (StateReturnCallback<T>, TransitionsCallback<T>)
}
impl<T: Dispatcher> ActiveState<T>
{
    fn into_unactive(self) -> UnactiveState<T>
    {
        UnactiveState
        {
            id: self.id,
            state: self.to_unactive.0,
            transitions: self.to_unactive.1,
        }
    }

    fn avaiable_transition(&self) -> Option<&Transition<T>>
    {
        self.transitions.iter().find(|transition| (transition.0)(&self.state))
    }
}

/// unactive states holds data to be able to reactivate itself
pub struct UnactiveState<T>
{
    id : StateId,
    state : fn() -> T,
    transitions : fn() -> Vec<Transition<T>>
}

impl<T: Dispatcher> UnactiveState<T>
{
    #[inline]
    fn into_active(self) -> ActiveState<T>
    {
        ActiveState
        {
            id: self.id,
            state: (self.state)(),
            event: StateEvent::default(),
            transitions: (self.transitions)(),
            to_unactive: (self.state, self.transitions),
        }
    }
}

pub struct Fsm<T, U: Dispatcher>
{
    // this will be the first state that enters the statemachine
    current : T,
    // vec of unactive states so that we dont instantiate anything that calls on uninitialized application data 
    states : ahash::AHashMap<StateId, UnactiveState<U>>,
}

impl<T: Dispatcher> Default for Fsm<UnactiveState<T>, T>
{
    fn default() -> Self 
    {
        Self::new()
    }
}

impl<T: Dispatcher> Fsm<UnactiveState<T>, T>
{
    /// initializes the builder with empty values
    pub fn new() -> Self
    {
        Self
        {
            current : UnactiveState
            {
                id : <Empty as State<T>>::id(),
                transitions: Vec::new,
                state: <Empty>::into_dispatch
            },
            states: ahash::AHashMap::new()
        }
    }

    /// adds a state to the fsm
    /// 
    /// # panics
    /// 
    /// panics if the state was already added 
    pub fn add_state<St>(&mut self, transitions: fn() -> Vec<Transition<T>>)
    where
        St: State<T> + 'static
    {
        debug_assert!
        (
            !self.states.contains_key(&<St>::id()) && self.current.id != <St>::id(),
            "attempted to add the same state twice 👮"
        );

        let state = UnactiveState::<T>
        {
            id: <St>::id(),
            state: <St>::into_dispatch,
            transitions,
        };

        match self.current.id != <Empty as State<T>>::id()
        {
            true => { self.states.insert(<St>::id(), state); }
            false => self.current = state
        }
    }

    pub fn build(self) -> Fsm<ActiveState<T>,T>
    {
        Fsm { current: self.current.into_active(), states : self.states }
    }

    pub fn is_empty(&self) -> bool
    {
        self.states.is_empty()
    }  
}

impl<T : Dispatcher> Fsm<ActiveState<T>, T>
{
    #[inline]
    pub fn update(&mut self)
    {
        self.current.state.dispatch(&self.current.event);

        match &self.current.event
        {  
            StateEvent::Update => if let Some(transition) = self.current.avaiable_transition()
            {
                self.current.event = StateEvent::Exit(transition.1)
            }
            
            StateEvent::Enter => self.current.event = StateEvent::Update,
    
            StateEvent::Exit(id) =>
            {
                let new = self.states.remove(id)
                .expect("attempted to transition to a state which wasn't found, probably because it wasn't added to the fsm 👮")
                .into_active();

                let old = std::mem::replace(&mut self.current, new).into_unactive();
                self.states.insert(old.id, old);
            }
        }
    }
}

