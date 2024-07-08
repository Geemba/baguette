use crate::*;

/// a state of this application
pub trait AppState
{
    /// describe how to initialize the implementor 
    fn new(app: &mut App) -> Self where Self: Sized;

    fn update(&mut self, app: &mut App, event: &StateEvent);

    fn transitions(&self, _: &App) -> Option<StateId>
    {
        None
    }

    fn id() -> StateId where Self: Sized + 'static
    {
        StateId::of::<Self>()
    }
}

/// an empty state to use as default
impl AppState for Dummy
{
    fn new(_ : &mut App) -> Self
    {
        Dummy
    }

    fn update(&mut self, _: &mut App, _: &StateEvent) {}
}

pub struct ActiveState
{
    id: StateId,
    state: Box<dyn AppState>,
    getter: fn(&mut App) -> Box<dyn AppState>,
    /// dictates which event will be called in the update function
    event: StateEvent,
}

impl ActiveState
{
    fn into_unactive(self) -> UnactiveState
    {
        UnactiveState { activator: self.getter, id: self.id }
    }

    fn avaiable_transition(&mut self, app: &mut App) -> Option<StateId>
    {
        self.state.transitions(app)
    }
}

/// unactive states holds data to be able to reactivate themselves
pub struct UnactiveState
{
    activator: fn(&mut App) -> Box<dyn AppState>,
    id: StateId
}

impl UnactiveState
{
    #[inline]
    fn into_active(self, application: &mut App) -> ActiveState
    {
        ActiveState
        {
            state: (self.activator)(application),
            event: StateEvent::default(),
            getter: self.activator,
            id: self.id,
        }
    }
}

pub struct FsmData<T>
{
    // this will be the first state that enters the statemachine
    current: T,
    // vec of unactive states so that we dont instantiate anything that calls on uninitialized application data 
    states: ahash::AHashMap<StateId, UnactiveState>
}

impl Default for FsmData<UnactiveState>
{
    fn default() -> Self 
    {
        Self
        {
            current: UnactiveState { activator: |_| Box::new(Dummy), id: StateId::default()},
            states: ahash::AHashMap::new()
        }
    }
}

impl FsmData<UnactiveState>
{
    /// adds a state to the fsm
    /// 
    /// # panics
    /// 
    /// panics if the state was already added 
    pub fn add_state<T>(&mut self)
    where
        T: AppState + 'static
    {
        let id = StateId::of::<T>();

        let state = UnactiveState
        {
            id, activator: |app| Box::new(T::new(app))
        };

        match self.current.id != StateId::default()
        {
            true =>
            {
                self.states.insert(id, state);
            }
            false => self.current = state
        }
    }

    pub fn build(self, app: &mut App) -> FsmData<ActiveState>
    {
        FsmData
        {
            current: self.current.into_active(app), states: self.states
        }
    }

    pub fn is_empty(&self) -> bool
    {
        self.states.is_empty()
    }
}

impl FsmData<ActiveState>
{
    #[inline]
    pub fn update(&mut self, app: &mut App)
    {
        self.current.state.update(app, &self.current.event);

        match &self.current.event
        {  
            StateEvent::Update => if let Some(state_id) = self.current.avaiable_transition(app)
            {
                self.current.event = StateEvent::Exit(state_id)
            }
            StateEvent::Enter => self.current.event = StateEvent::Update,

            StateEvent::Exit(id) =>
            {
                let new = self.states.remove(id)
                .expect
                (
                    "attempted to transition to a state that wasn't found,
                    probably because it wasn't added to the fsm"
                )
                .into_active(app);

                let old = std::mem::replace(&mut self.current, new);

                self.states.insert(old.id, old.into_unactive());
            }
        }
    }
}

