pub trait OnBeforeScreenRedraw : Sync
{
    fn on_before_screen_redraw(&mut self);
}

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

/// [Transition] is composed of a predicate function and the [StateId]
/// it should transition to if the predicate returns true
pub type Transition = (fn(&dyn State) -> bool, StateId);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[derive(Debug)]
pub struct StateId(core::any::TypeId);

pub type UnactiveStateMachine = Fsm<UnactiveState>;
pub type StateMachine = Fsm<ActiveState>;

pub struct ActiveState
{
    id : StateId,
    state : Box<dyn State>,
    event : StateEvent,
    transitions : Vec<(fn(&dyn State) -> bool, StateId)>,
    to_unactive : (fn() -> Box<dyn State>, fn() -> Vec<Transition>)
}
impl ActiveState
{
    fn to_unactive(self) -> UnactiveState
    {
        UnactiveState
        {
            id: self.id,
            state: self.to_unactive.0,
            transitions: self.to_unactive.1,
        }
    }

    fn avaiable_transition(&self) -> Option<&Transition>
    {
        self.transitions.iter().find(|transition| (transition.0)(&*self.state))
    }
}

/// unactive states holds data to be able to reactivate itself
pub struct UnactiveState
{
    id : StateId,
    state : fn() -> Box<dyn State>,
    transitions : fn() -> Vec<Transition>
}

impl UnactiveState
{
    #[inline]
    fn to_active(self) -> ActiveState
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

pub trait State : Sync
{
    fn new() -> Self where Self : Sized;

    fn update(&mut self, event : &StateEvent);

    fn id() -> StateId
    where
        Self: core::any::Any + Sized
    {
        StateId(core::any::TypeId::of::<Self>())
    }
}

struct Empty;

fn empty_id() -> StateId {Empty::id()}

/// an empty state to use as default
impl State for Empty
{
    fn new() -> Self where Self : Sized {Self}
    fn update(&mut self, _ : &StateEvent) {}
}

impl State for ()
{
    fn new() where Self : Sized {}
    fn update(&mut self, _ : &StateEvent) {}
}

pub struct Fsm<T>
{
    // this will be the first state that enters the statemachine
    current : T,
    // vec of unactive states so that we dont instantiate anything that calls on uninitialized application data 
    states : ahash::AHashMap<StateId, UnactiveState>
}

impl Fsm<UnactiveState>
{
    /// initializes the builder with empty values
    pub fn new() -> Self
    {
        Self
        { 
            current : UnactiveState
            {
                id : <Empty>::id(),
                transitions: Vec::new,
                state: || Box::new(Empty),
            },
            states: ahash::AHashMap::new()
        }
    }

    /// adds a state to the fsm
    /// 
    /// # panics
    /// 
    /// panics if the state was already added 
    pub fn add_state<St>(&mut self, transitions: fn() -> Vec<(fn(&dyn State) -> bool, StateId)>)
    where
        St: State + 'static,
    {
        debug_assert!
        (
            !self.states.contains_key(&<St>::id())
            && self.current.id != <St>::id(),
            "attempted to add a state twice"
        );

        let state = UnactiveState
        {
            id: <St>::id(),
            state: || Box::new(St::new()),
            transitions,
        };

        match self.current.id != empty_id()
        {
            true => {self.states.insert(<St>::id(), state);}
            false => self.current = state
        }
    }

    pub fn build(self) -> Fsm<ActiveState>
    {
        Fsm{ current: self.current.to_active(), states : self.states }
    }
}

impl Fsm<ActiveState>
{
    #[inline]
    pub fn update(&mut self)
    {
        self.current.state.update(&self.current.event);

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
                .expect("attempted to transition to a state which wasn't found")
                .to_active();

                let old = std::mem::replace(&mut self.current, new).to_unactive();
                self.states.insert(old.id, old);
            }
        }
    }
}
#[macro_export]
/// changes state if the predicate returns `true` 
macro_rules! transitions
{   
    [$($lbracket:tt $closure:ident $rbracket:tt $predicate:expr => $type:ident),*] =>
    {
       || vec![$(($lbracket $closure $rbracket $predicate, $type::id())),*]
    };
}