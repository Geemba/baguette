//! # baguette-input
//! baguette's input module

use winit::dpi::PhysicalSize;
pub use winit::event::{*, VirtualKeyCode as KeyCode};
pub use winit;

    // callbacks to application events
    static mut ON_SCREEN_RESIZE : once_cell::sync::Lazy<Callback<PhysicalSize<u32>>> = once_cell::sync::Lazy::new(|| Callback::new());

    /// callback to execute a function when the window is resized
    /// 
    /// you can attach a callback by using implementing the [CallbackListener] trait
    pub fn on_screen_resize() -> &'static mut Callback<PhysicalSize<u32>>
    {
        unsafe { &mut ON_SCREEN_RESIZE }
    }

/// the input system of the engine
pub struct Input
{
    current_pressed_keys : ahash::AHashMap<KeyCode, State>,
    pressed_mouse_buttons : ahash::AHashMap<MouseButton, State>,
}

static mut INPUT : std::sync::OnceLock<Input> = std::sync::OnceLock::new();

pub fn input() -> &'static Input
{
    unsafe { INPUT.get() }.expect("input checking is not initialized")
}

pub fn input_mut() -> &'static mut Input
{
    unsafe { INPUT.get_mut() }.expect("input checking is not initialized")
}

/// returns `true` if the key is being pressed
pub fn get_key_holding(keycode : KeyCode) -> bool
{
    input().current_pressed_keys.get(&keycode).is_some()
}

/// returns `true` the frame the key is released
pub fn get_key_up(keycode : KeyCode) -> bool
{
    match input().current_pressed_keys.get(&keycode)
    {
        Some(state) => state.released,
        None => false
    }
}

/// returns `true` the first frame the button is pressed,
pub fn get_key_down(keycode : KeyCode) -> bool
{
    let state = input_mut().current_pressed_keys.get_mut(&keycode);

    match state
    {
        Some(State { pressed_this_frame : true, .. }) =>
        {
            // theres a bunch of frame delay before the program checks the input again,
            // lets change this ourselves to false since it will definitely be after the first fn invocation
            state.unwrap().pressed_this_frame = false;
            true
        }

        _ => false
    }
}

/// represents the current state of an active input
struct State
{
    pressed_this_frame : bool,
    released : bool,
}

impl Input
{
    pub fn init() -> &'static mut Self
    {
        match unsafe { INPUT.set(Input::new()) }
        {
            Ok(()) => unsafe { INPUT.get_mut() }.unwrap(),
            Err(_) => panic!("input was already initialized"),
        }  
    }

    pub fn new() -> Self
    {
        Self
        {
            current_pressed_keys : ahash::AHashMap::with_capacity(8),
            pressed_mouse_buttons : ahash::AHashMap::with_capacity(3),
        }
    }

    #[inline]
    pub fn check(&mut self, event: &WindowEvent)
    {    
        match event
        {
            WindowEvent::KeyboardInput { input : KeyboardInput { state, virtual_keycode : Some(key), .. }, .. } =>
            {
                match state
                {
                    ElementState::Pressed =>
                    {
                        if let None = self.current_pressed_keys.get(key)
                        {
                            self.current_pressed_keys.insert(*key, State { pressed_this_frame: true, released: false });
                        }
                    }
                    ElementState::Released =>
                    {
                        if let Some(State { released, .. }) = self.current_pressed_keys.get_mut(key)
                        {
                            *released = true;
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } =>
            {
                match state
                {
                    ElementState::Pressed =>
                    {
                        if let None = self.pressed_mouse_buttons.get(button)
                        {
                            self.pressed_mouse_buttons.insert(*button, State { pressed_this_frame: true, released: false });
                        }
                    }
                    ElementState::Released =>
                    {
                        if let Some(State { released, .. }) = self.pressed_mouse_buttons.get_mut(button)
                        {
                            *released = true;
                        }
                    }
                }
            }
            _ => (),
        }
    }
    
    #[allow(dead_code)]
    pub fn flush_released_keys(&mut self)
    {
        self.current_pressed_keys.retain(|_,state| !state.released);
        self.pressed_mouse_buttons.retain(|_,state| !state.released);
    }

    /// returns true the first frame the button is pressed,
    pub fn get_key_down(keycode : KeyCode) -> bool
    {
        let state = input_mut().current_pressed_keys.get_mut(&keycode);

        match state
        {
            Some(State { pressed_this_frame : true, .. }) =>
            {
                // theres a bunch of frame delay before the program checks the input again,
                // lets change this ourselves to false since it will definitely be after the first fn invocation
                state.unwrap().pressed_this_frame = false;
                true
            }

            _ => false
        }
    }

    /// returns true if the key is being pressed
    pub fn get_key_holding(keycode : KeyCode) -> bool
    {
        input().current_pressed_keys.get(&keycode).is_some()
    }

    /// returns true the frame the key is released
    pub fn get_key_up(keycode : KeyCode) -> bool
    {
        match input().current_pressed_keys.get(&keycode)
        {
            Some(state) => state.released,
            None => false
        }
    }

    // returns true the first frame the mouse button is pressed

    pub fn get_mouse_button_down(click : MouseButton) -> bool
    {
        let state = input_mut().pressed_mouse_buttons.get_mut(&click);

        match state
        {
            Some(State { pressed_this_frame : true, .. }) =>
            {
                // theres a bunch of frame delay before the program checks the input again,
                // lets change this ourselves to false since it will definitely be after the first fn invocation
                state.unwrap().pressed_this_frame = false;
                true
            }

            _ => false
        }
    }

    pub fn get_mouse_button_holding(click : MouseButton) -> bool
    {
        input().pressed_mouse_buttons.get(&click).is_some()
    }

    pub fn get_mouse_button_up(click : MouseButton) -> bool
    {
        match input().pressed_mouse_buttons.get(&click)
        {
            Some(input) => input.released,
            None => false
        }  
    }
}

pub struct Callback<T:'static + Copy>
{
    listeners : Vec<&'static mut dyn CallbackListener<T>>
}

impl<T: Copy> core::ops::AddAssign<&mut dyn CallbackListener<T>> for &mut Callback<T>
{
    fn add_assign(&mut self, callback: &mut dyn CallbackListener<T>)
    {
        self.add_listener(callback)
    }
}

impl<T: Copy> Callback<T>
{
    pub fn new() -> Self { Self { listeners : vec![] } }

    pub fn add_listener(&mut self, callback : &mut dyn CallbackListener<T>)
    {
        self.listeners.push(unsafe { core::mem::transmute(callback) });
    }

    pub fn invoke(&mut self, param : T)
    {
        for listener in self.listeners.iter_mut()
        {
            listener.callback_listener(param)
        }
    }
}

pub trait CallbackListener<ParamType>
{
    fn callback_listener(&mut self, t : ParamType);
}