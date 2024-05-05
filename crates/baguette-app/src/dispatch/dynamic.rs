//! dynamically dispatched
use crate::*;

/// [Transition] is composed of a `predicate` function and the [StateId]
/// it should transition to if the predicate returns `true`
pub type Transition = (fn(&DispatchedState) -> bool, StateId);

/// how the state type is dispatched 
type DispatchedState = Box<dyn State>;
/// a function pointer that returns the state concrete type
type StateReturnCallback = fn(&mut App) -> DispatchedState;
/// a function pointer that returns a vec containing all the possible transitions
type TransitionsCallback = fn() -> Vec<Transition>;

pub trait State
{
    fn new(app: &mut App) -> Self where Self: Sized;

    fn update(&mut self, app: &mut App, event: &StateEvent);

    fn id() -> StateId
    where
        Self: core::any::Any + Sized
    {
        StateId(core::any::TypeId::of::<Self>())
    }
}

/// an empty state to use as default
impl State for Empty
{
    fn new(_ : &mut App) -> Self where Self : Sized
    {
        Empty
    }

    fn update(&mut self, _:&mut App, _ : &StateEvent) {}
}

impl State for ()
{
    fn new(_ : &mut App) where Self : Sized {}
    fn update(&mut self, _:&mut App, _ : &StateEvent) {}
}

pub struct ActiveState
{
    id: StateId,
    state: DispatchedState,
    event: StateEvent,
    transitions: Vec<Transition>,
    to_unactive: (StateReturnCallback, TransitionsCallback)
}
impl ActiveState
{
    fn into_unactive(self) -> UnactiveState
    {
        UnactiveState
        {
            id: self.id,
            state: self.to_unactive.0,
            transitions: self.to_unactive.1,
        }
    }

    fn avaiable_transition(&mut self) -> Option<&Transition>
    {
        self.transitions.iter().find(|transition| (transition.0)(&self.state))
    }
}

/// unactive states holds data to be able to reactivate themselves
pub struct UnactiveState
{
    id: StateId,
    state: fn(&mut App) -> DispatchedState,
    transitions: fn() -> Vec<Transition>
}

impl UnactiveState
{
    #[inline]
    fn into_active(self, application: &mut App) -> ActiveState
    {
        ActiveState
        {
            id: self.id,
            state: (self.state)(application),
            event: StateEvent::default(),
            transitions: (self.transitions)(),
            to_unactive: (self.state, self.transitions),
        }
    }
}

pub struct Fsm<T>
{
    // this will be the first state that enters the statemachine
    current : T,
    // vec of unactive states so that we dont instantiate anything that calls on uninitialized application data 
    states : ahash::AHashMap<StateId, UnactiveState>
}

impl Default for Fsm<UnactiveState>
{
    fn default() -> Self 
    {
        Self
        {
            current : UnactiveState
            {
                id : <Empty as State>::id(),
                transitions: Vec::new,
                state: |_| Box::new(())
            },
            states: ahash::AHashMap::new()
        }
    }
}

impl Fsm<UnactiveState>
{
    /// adds a state to the fsm
    /// 
    /// # panics
    /// 
    /// panics if the state was already added 
    pub fn add_state<St>(&mut self, transitions: fn() -> Vec<Transition>)
    where
        St: State + 'static
    {
        assert!
        (
            !self.states.contains_key(&<St>::id()) && self.current.id != <St>::id(),
            "attempted to add the same state twice 👮"
        );

        let state = UnactiveState
        {
            id: <St>::id(),
            transitions,
            state: |app| Box::new(<St>::new(unsafe { core::mem::transmute(app) }))
        };

        match self.current.id != <Empty as State>::id()
        {
            true => { self.states.insert(<St>::id(), state); }
            false => self.current = state
        }
    }

    pub fn build(self, app: &mut App) -> Fsm<ActiveState>
    {
        Fsm
        {
            current: self.current.into_active(app), states : self.states
        }
    }

    pub fn is_empty(&self) -> bool
    {
        self.states.is_empty()
    }
}

impl Fsm<ActiveState>
{
    #[inline]
    pub fn update(&mut self, app: &mut App)
    {
        self.current.state.update(app, &self.current.event);

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
                .into_active(app);

                let old = std::mem::replace(&mut self.current, new).into_unactive();
                self.states.insert(old.id, old);
            }
        }
    }
}

