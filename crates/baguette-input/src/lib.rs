//! # baguette-input
//! baguette's input module

use winit::keyboard::PhysicalKey;

pub use winit::event::*;
pub use winit::keyboard::KeyCode;

pub use winit;

pub static mut INPUT: once_cell::sync::OnceCell<Input> = once_cell::sync::OnceCell::new();

/// returns `true` the first frame the key is pressed
/// 
/// use [get_key_holding] to check if the key is pressed in the current frame
pub fn get_key_down(keycode: KeyCode) -> bool
{
    unsafe { INPUT.get_mut().unwrap().get_key_down(keycode) }
}

/// returns `true` if the key is pressed in the current frame
pub fn get_key_holding(keycode: KeyCode) -> bool
{
    unsafe { INPUT.get_mut().unwrap().get_key_holding(keycode) }
}

/// returns `true` when the key is released
/// 
/// use [get_key_holding] to check if the key is pressed in the current frame
pub fn get_key_up(keycode: KeyCode) -> bool
{
    unsafe { INPUT.get_mut().unwrap().get_key_up(keycode) }
}

pub fn get_mouse_button_down(button: MouseButton) -> bool
{
    unsafe { INPUT.get_mut().unwrap().get_mouse_button_down(button) } 
}

pub fn get_mouse_button_holding(button: MouseButton) -> bool
{
    unsafe { INPUT.get().unwrap().get_mouse_button_holding(button) } 
}

pub fn get_mouse_button_up(button: MouseButton) -> bool
{
    unsafe { INPUT.get().unwrap().get_mouse_button_up(button) } 
}

pub fn input_axis() -> baguette_math::Vec2
{
    baguette_math::Vec2::new(horizontal_axis(), vertical_axis())
}


pub fn mouse_position() -> baguette_math::Vec2
{
    unsafe { INPUT.get_mut().unwrap().mouse_position() } 
}

pub fn horizontal_axis() -> f32
{
    let mut x = 0.;

    x -= match get_key_holding(KeyCode::KeyA)
    {
        true => 1.,
        false => 0.
    };
    x += match get_key_holding(KeyCode::KeyD)
    {
        true => 1.,
        false => 0.
    };

    x
}

pub fn vertical_axis() -> f32
{
    let mut y = 0.;

    y -= match get_key_holding(KeyCode::KeyW)
    {
        true => 1.,
        false => 0.
    };
    y += match get_key_holding(KeyCode::KeyS)
    {
        true => 1.,
        false => 0.
    };

    y
}

#[derive(Default)]
/// the input system of the engine
pub struct Input
{
    current_pressed_keys: ahash::AHashMap<PhysicalKey, InputState>,
    pressed_mouse_buttons: ahash::AHashMap<MouseButton, InputState>,
    cursor_position: baguette_math::Vec2
}

/// holds the current state of an active input
struct InputState
{
    pressed_this_frame: bool,
    released: bool
}

impl Input
{
    /// initialized the input checker or returns a reference to it
    pub fn init() -> &'static mut Self
    {
        unsafe 
        {         
            INPUT.get_or_init(Self::default);
            INPUT.get_mut().unwrap()
        }
    }

    #[inline]
    pub fn check(&mut self, event: &WindowEvent)
    {    
        match event
        {
            WindowEvent::KeyboardInput{ event: KeyEvent { physical_key, state,.. }, .. } =>
            {
                match state
                {
                    ElementState::Pressed =>
                    {
                        if self.current_pressed_keys.get(physical_key).is_none()
                        {
                            self.current_pressed_keys.insert
                            (
                                *physical_key,
                                InputState { pressed_this_frame: true, released: false }
                            );
                        }
                    }
                    ElementState::Released =>
                    {
                        if let Some(InputState { released, .. }) = self.current_pressed_keys.get_mut(physical_key)
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
                        if self.pressed_mouse_buttons.get(button).is_none()
                        {
                            self.pressed_mouse_buttons.insert
                            (
                                *button, InputState { pressed_this_frame: true, released: false }
                            );
                        }
                    }
                    ElementState::Released =>
                    {
                        if let Some(InputState { released, .. }) = self.pressed_mouse_buttons.get_mut(button)
                        {
                            *released = true;
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } =>
            {
                self.cursor_position = baguette_math::Vec2::new
                (
                    position.x as f32, position.y  as f32
                )
            }
            _ => (/*ignore other events*/)
        }
    }
    
    #[allow(dead_code)]
    pub fn flush_released_keys(&mut self)
    {
        self.current_pressed_keys.retain(|_,state| !state.released);
        self.pressed_mouse_buttons.retain(|_,state| !state.released);
    }
}

// keys
impl Input
{
    /// returns true the first frame the button is pressed,
    pub fn get_key_down(&mut self, keycode : KeyCode) -> bool
    {
        let state = self.current_pressed_keys.get_mut(&PhysicalKey::Code(keycode));

        match state
        {
            Some(InputState { pressed_this_frame: true, .. }) =>
            {
                // theres a bunch of frame delay before the program checks the input again,
                // lets change this ourselves to false since it will definitely be after the first invocation
                state.unwrap().pressed_this_frame = false;
                true
            }

            _ => false
        }
    }

    /// returns true if the key is being pressed
    pub fn get_key_holding(&self, keycode: KeyCode) -> bool
    {
        self.current_pressed_keys.get(&PhysicalKey::Code(keycode)).is_some()
    }

    /// returns true the frame the key is released
    pub fn get_key_up(&self, keycode: KeyCode) -> bool
    {
        match self.current_pressed_keys.get(&PhysicalKey::Code(keycode))
        {
            Some(state) => state.released,
            None => false
        }
    }
}

// mouse
impl Input
{
    // returns true the first frame the mouse button is pressed
    pub fn get_mouse_button_down(&mut self, click: MouseButton) -> bool
    {
        let state = self.pressed_mouse_buttons.get_mut(&click);

        match state
        {
            Some(InputState { pressed_this_frame: true, .. }) =>
            {
                // theres a bunch of frame delay before the program checks the input again,
                // lets change this ourselves to false since it will definitely be after the first fn invocation
                state.unwrap().pressed_this_frame = false;
                true
            }

            _ => false
        }
    }

    pub fn get_mouse_button_holding(&self, click: MouseButton) -> bool
    {
        self.pressed_mouse_buttons.get(&click).is_some()
    }

    pub fn get_mouse_button_up(&self, click: MouseButton) -> bool
    {
        match self.pressed_mouse_buttons.get(&click)
        {
            Some(input) => input.released,
            None => false
        }  
    }
    
    fn mouse_position(&self) -> baguette_math::Vec2
    {
        self.cursor_position
    }
}

//pub struct Callback<T:'static + Copy>
//{
//    listeners: Vec<&'static mut dyn CallbackListener<T>>
//}
//
//impl<T: 'static + Copy> Default for Callback<T> 
//{
//    fn default() -> Self 
//    {
//        Self { listeners : vec![] }
//    }
//}
//
//impl<T: Copy> core::ops::AddAssign<&mut dyn CallbackListener<T>> for &mut Callback<T>
//{
//    fn add_assign(&mut self, callback: &mut dyn CallbackListener<T>)
//    {
//        self.add_listener(callback)
//    }
//}
//
//impl<T: Copy> Callback<T>
//{
//    pub fn add_listener(&mut self, callback: &mut dyn CallbackListener<T>)
//    {
//        self.listeners.push(unsafe { core::mem::transmute(callback) });
//    }
//
//    pub fn invoke(&mut self, param : T)
//    {
//        for listener in self.listeners.iter_mut()
//        {
//            listener.callback_listener(param)
//        }
//    }
//}
//
//pub trait CallbackListener<ParamType>
//{
//    fn callback_listener(&mut self, t : ParamType);
//}