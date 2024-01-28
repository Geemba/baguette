//! # baguette-input
//! baguette's input module

use winit::keyboard::PhysicalKey;

pub use winit::event::*;
pub use winit::keyboard::KeyCode;

pub use winit;

#[derive(Default)]
/// the input system of the engine, this is managed by the engine
pub struct InputHandler
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

impl InputHandler
{
    pub fn check(&mut self, event: &WindowEvent)
    {
        match event
        {
            WindowEvent::KeyboardInput{ event: KeyEvent { physical_key, state,.. }, .. } =>
            {
                if let ElementState::Pressed = state
                {
                    match self.current_pressed_keys.get_mut(physical_key)
                    {
                        Some(state) => state.pressed_this_frame = false,
                        None => 
                        {
                            self.current_pressed_keys.insert
                            (
                                *physical_key,
                                InputState { pressed_this_frame: true, released: false }
                            );
                        }
                    }
                }
                else 
                {
                    if let Some(InputState { released, .. }) = self.current_pressed_keys.get_mut(physical_key)
                    {
                        *released = true;
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } =>
            {
                match state
                {
                    ElementState::Pressed =>
                    
                        if self.pressed_mouse_buttons.get(button).is_none()
                        {
                            self.pressed_mouse_buttons.insert
                            (
                                *button, InputState { pressed_this_frame: true, released: false }
                            );
                        }
                    
                    ElementState::Released =>
                    
                        if let Some(InputState { released, .. }) = self.pressed_mouse_buttons.get_mut(button)
                        {
                            *released = true;
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
    
    pub fn flush_released_keys(&mut self)
    {
        self.current_pressed_keys.retain(|_,state| !state.released);
        self.pressed_mouse_buttons.retain(|_,state| !state.released);
        
        self.current_pressed_keys
        .iter_mut()
        .for_each
        (
            |(..,state)| if state.pressed_this_frame
            {
                state.pressed_this_frame = false
            }
        );

        self.pressed_mouse_buttons
        .iter_mut()
        .for_each
        (
            |(..,state)| if state.pressed_this_frame
            {
                state.pressed_this_frame = false
            }
        );
    }
}

pub struct Input<'a>
{
    handler: &'a InputHandler
}

impl<'a> From<&'a InputHandler> for Input<'a>
{
    fn from(handler: &'a InputHandler) -> Self { Self { handler } }
}

// keys
impl Input<'_>
{
    /// returns true the first frame the button is pressed,
    pub fn get_key_down(&self, keycode: KeyCode) -> bool
    {
        self.handler.current_pressed_keys
            .get(&PhysicalKey::Code(keycode))
            .is_some_and(|key| key.pressed_this_frame == true)
    }

    /// returns true if the key is being pressed
    pub fn get_key_holding(&self, keycode: KeyCode) -> bool
    {
        self.handler.current_pressed_keys.get(&PhysicalKey::Code(keycode)).is_some()
    }

    /// returns true the frame the key is released
    pub fn get_key_up(&self, keycode: KeyCode) -> bool
    {
        match self.handler.current_pressed_keys.get(&PhysicalKey::Code(keycode))
        {
            Some(state) => state.released,
            None => false
        }
    }

    /// the horizontal input axis, can be anything between -1 and 1
    pub fn horizontal_axis(&self) -> f32
    {
        let mut x = 0.;

        x -= match self.get_key_holding(KeyCode::KeyA)
        {
            true => 1.,
            false => 0.
        };
        x += match self.get_key_holding(KeyCode::KeyD)
        {
            true => 1.,
            false => 0.
        };

        x
    }

    /// the vertical input axis, can be anything between -1 and 1
    pub fn vertical_axis(&self) -> f32
    {
        let mut y = 0.;

        y -= match self.get_key_holding(KeyCode::KeyW)
        {
            true => 1.,
            false => 0.
        };
        y += match self.get_key_holding(KeyCode::KeyS)
        {
            true => 1.,
            false => 0.
        };

        y
    }

    pub fn input_axis(&self) -> baguette_math::Vec2
    {
        baguette_math::Vec2::new(self.horizontal_axis(), self.vertical_axis())
    }
}

// mouse
impl Input<'_>
{
    // returns true the first frame the mouse button is pressed
    pub fn get_mouse_button_down(&self, click: MouseButton) -> bool
    {
        self.handler.pressed_mouse_buttons
            .get(&click)
            .is_some_and(|button| button.pressed_this_frame == true)

        //match state
        //{
        //    Some(InputState { pressed_this_frame: true, .. }) =>
        //    {
        //        // theres a bunch of frame delay before the program checks the input again,
        //        // lets change this ourselves to false since it will definitely be after the first fn invocation
        //        state.unwrap().pressed_this_frame = false;
        //        true
        //    }

        //    _ => false
        //}
    }

    #[inline]
    pub fn get_mouse_button_holding(&self, click: MouseButton) -> bool
    {
        self.handler.pressed_mouse_buttons.get(&click).is_some()
    }

    #[inline]
    pub fn get_mouse_button_up(&self, click: MouseButton) -> bool
    {
        match self.handler.pressed_mouse_buttons.get(&click)
        {
            Some(input) => input.released,
            None => false
        }  
    }
    
    #[inline]
    pub fn mouse_position(&self) -> baguette_math::Vec2
    {
        self.handler.cursor_position
    }
}