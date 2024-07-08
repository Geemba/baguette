//! [`egui`] bindings for [`winit`](https://github.com/rust-windowing/winit).
//!
//! The library translates winit events to egui, handled copy/paste,
//! updates the cursor, open links clicked in egui, etc.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::manual_range_contains)]

#[cfg(feature = "accesskit")]
pub use accesskit_winit;
pub use egui;
#[cfg(feature = "accesskit")]
use egui::accesskit;
use egui::{Pos2, Rect, Vec2, ViewportCommand, ViewportId};
pub use input::winit;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::{CursorGrabMode, Window, WindowButtons, WindowLevel},
};

pub fn screen_size_in_pixels(window: &Window) -> egui::Vec2 {
    let size = window.inner_size();
    egui::vec2(size.width as f32, size.height as f32)
}

/// Calculate the `pixels_per_point` for a given window, given the current egui zoom factor
pub fn pixels_per_point(egui_ctx: &egui::Context, window: &Window) -> f32 {
    let native_pixels_per_point = window.scale_factor() as f32;
    let egui_zoom_factor = egui_ctx.zoom_factor();
    egui_zoom_factor * native_pixels_per_point
}

// ----------------------------------------------------------------------------

#[must_use]
#[derive(Clone, Copy, Debug, Default)]
pub struct EventResponse {
    /// If true, egui consumed this event, i.e. wants exclusive use of this event
    /// (e.g. a mouse click on an egui window, or entering text into a text field).
    ///
    /// For instance, if you use egui for a game, you should only
    /// pass on the events to your game when [`Self::consumed`] is `false.
    ///
    /// Note that egui uses `tab` to move focus between elements, so this will always be `true` for tabs.
    pub consumed: bool,

    /// Do we need an egui refresh because of this event?
    pub repaint: bool,
}

// ----------------------------------------------------------------------------

/// Handles the integration between egui and a winit Window.
///
/// Instantiate one of these per viewport/window.
pub struct State {
    /// Shared clone.
    pub ctx: egui::Context,

    viewport_id: ViewportId,
    start_time: std::time::Instant,
    input: egui::RawInput,
    pointer_pos_in_points: Option<egui::Pos2>,
    any_pointer_button_down: bool,
    current_cursor_icon: Option<egui::CursorIcon>,

    /// If `true`, mouse inputs will be treated as touches.
    /// Useful for debugging touch support in egui.
    ///
    /// Creates duplicate touches, if real touch inputs are coming.
    simulate_touch_screen: bool,

    /// Is Some(…) when a touch is being translated to a pointer.
    ///
    /// Only one touch will be interpreted as pointer at any time.
    pointer_touch_id: Option<u64>,

    /// track ime state
    input_method_editor_started: bool,

    #[cfg(feature = "accesskit")]
    accesskit: Option<accesskit_winit::Adapter>,

    //allow_ime: bool,
}

impl State {
    /// Construct a new instance
    pub fn new(
        native_pixels_per_point: Option<f32>,
        max_texture_side: Option<usize>,
    ) -> Self {

        let input = egui::RawInput {
            focused: true, // winit will tell us when we have focus
            ..Default::default()
        };

        let mut this = Self {
            ctx: Default::default(),
            viewport_id: egui::ViewportId::ROOT,
            start_time: std::time::Instant::now(),
            input,
            pointer_pos_in_points: None,
            any_pointer_button_down: false,
            current_cursor_icon: None,

            simulate_touch_screen: false,
            pointer_touch_id: None,

            input_method_editor_started: false,

            #[cfg(feature = "accesskit")]
            accesskit: None,

            //allow_ime: false,
        };

        this.input
            .viewports
            .entry(ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = native_pixels_per_point;

        if let Some(max_texture_side) = max_texture_side {
            this.set_max_texture_side(max_texture_side);
        }
        this
    }

    #[cfg(feature = "accesskit")]
    pub fn init_accesskit<T: From<accesskit_winit::ActionRequestEvent> + Send>(
        &mut self,
        window: &Window,
        event_loop_proxy: winit::event_loop::EventLoopProxy<T>,
        initial_tree_update_factory: impl 'static + FnOnce() -> accesskit::TreeUpdate + Send,
    ) {
        crate::profile_function!();
        self.accesskit = Some(accesskit_winit::Adapter::new(
            window,
            initial_tree_update_factory,
            event_loop_proxy,
        ));
    }

    /// Call this once a graphics context has been created to update the maximum texture dimensions
    /// that egui will use.
    pub fn set_max_texture_side(&mut self, max_texture_side: usize) {
        self.input.max_texture_side = Some(max_texture_side);
    }

    /// Prepare for a new frame by extracting the accumulated input,
    ///
    /// as well as setting [the time](egui::RawInput::time) and [screen rectangle](egui::RawInput::screen_rect).
    ///
    /// You need to set [`egui::RawInput::viewports`] yourself though.
    /// Use [`update_viewport_info`] to update the info for each
    /// viewport.
    pub fn take_egui_input(&mut self, window: &Window) -> egui::RawInput {

        self.input.time = Some(self.start_time.elapsed().as_secs_f64());
        
        // On Windows, a minimized window will have 0 width and height.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where egui window positions would be changed when minimizing on Windows.
        let screen_size_in_pixels = screen_size_in_pixels(window);
        let screen_size_in_points =
            screen_size_in_pixels / pixels_per_point(&self.ctx, window);

        self.input.screen_rect = (screen_size_in_points.x > 0.0
            && screen_size_in_points.y > 0.0)
            .then(|| Rect::from_min_size(Pos2::ZERO, screen_size_in_points));

        // Tell egui which viewport is now active:
        self.input.viewport_id = self.viewport_id;

        self.input
            .viewports
            .entry(self.viewport_id)
            .or_default()
            .native_pixels_per_point = Some(window.scale_factor() as f32);

        self.input.take()
    }

    /// Call this when there is a new event.
    ///
    /// The result can be found in [`Self::egui_input`] and be extracted with [`Self::take_egui_input`].
    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {

        #[cfg(feature = "accesskit")]
        if let Some(accesskit) = &self.accesskit {
            accesskit.process_event(window, event);
        }

        use winit::event::WindowEvent;
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let native_pixels_per_point = *scale_factor as f32;

                self.input
                    .viewports
                    .entry(self.viewport_id)
                    .or_default()
                    .native_pixels_per_point = Some(native_pixels_per_point);

                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_button_input(*state, *button);
                EventResponse {
                    repaint: true,
                    consumed: self.ctx.wants_pointer_input(),
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.on_mouse_wheel(window, *delta);
                EventResponse {
                    repaint: true,
                    consumed: self.ctx.wants_pointer_input(),
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.on_cursor_moved(window, *position);
                EventResponse {
                    repaint: true,
                    consumed: self.ctx.is_using_pointer(),
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.pointer_pos_in_points = None;
                self.input.events.push(egui::Event::PointerGone);
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            // WindowEvent::TouchpadPressure {device_id, pressure, stage, ..  } => {} // TODO
            WindowEvent::Touch(touch) => {
                self.on_touch(window, touch);
                let consumed = match touch.phase {
                    winit::event::TouchPhase::Started
                    | winit::event::TouchPhase::Ended
                    | winit::event::TouchPhase::Cancelled => self.ctx.wants_pointer_input(),
                    winit::event::TouchPhase::Moved => self.ctx.is_using_pointer(),
                };
                EventResponse {
                    repaint: true,
                    consumed,
                }
            }

            WindowEvent::Ime(ime) => {
                // on Mac even Cmd-C is pressed during ime, a `c` is pushed to Preedit.
                // So no need to check is_mac_cmd.
                //
                // How winit produce `Ime::Enabled` and `Ime::Disabled` differs in MacOS
                // and Windows.
                //
                // - On Windows, before and after each Commit will produce an Enable/Disabled
                // event.
                // - On MacOS, only when user explicit enable/disable ime. No Disabled
                // after Commit.
                //
                // We use input_method_editor_started to manually insert CompositionStart
                // between Commits.
                match ime {
                    winit::event::Ime::Enabled => self.input.events.push(egui::Event::Ime(egui::ImeEvent::Enabled)),
                    winit::event::Ime::Disabled => self.input.events.push(egui::Event::Ime(egui::ImeEvent::Disabled)),
                    winit::event::Ime::Commit(text) =>
                    {
                        self.input_method_editor_started = false;
                        self.input
                            .events
                            .push(egui::Event::Ime(egui::ImeEvent::Commit(text.clone())));
                    }
                    winit::event::Ime::Preedit(text, Some(_)) => {
                        if !self.input_method_editor_started {
                            self.input_method_editor_started = true;
                            self.input.events.push(egui::Event::Ime(egui::ImeEvent::Preedit(text.clone())));
                        }
                    }
                    winit::event::Ime::Preedit(_, None) => {}
                };

                EventResponse {
                    repaint: true,
                    consumed: self.ctx.wants_keyboard_input(),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // When pressing the Tab key, egui focuses the first focusable element, hence Tab always consumes.
                let consumed = self.on_keyboard_input(event)
                    || self.ctx.wants_keyboard_input()
                    || event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab);
                EventResponse {
                    repaint: true,
                    consumed,
                }
            }
            WindowEvent::Focused(focused) => {
                self.input.focused = *focused;
                // We will not be given a KeyboardInput event when the modifiers are released while
                // the window does not have focus. Unset all modifier state to be safe.
                self.input.modifiers = egui::Modifiers::default();
                self.input
                    .events
                    .push(egui::Event::WindowFocused(*focused));
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::HoveredFile(path) => {
                self.input.hovered_files.push(egui::HoveredFile {
                    path: Some(path.clone()),
                    ..Default::default()
                });
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::HoveredFileCancelled => {
                self.input.hovered_files.clear();
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::DroppedFile(path) => {
                self.input.hovered_files.clear();
                self.input.dropped_files.push(egui::DroppedFile {
                    path: Some(path.clone()),
                    ..Default::default()
                });
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::ModifiersChanged(state) => {
                let state = state.state();

                let alt = state.alt_key();
                let ctrl = state.control_key();
                let shift = state.shift_key();
                let super_ = state.super_key();

                self.input.modifiers.alt = alt;
                self.input.modifiers.ctrl = ctrl;
                self.input.modifiers.shift = shift;
                self.input.modifiers.mac_cmd = cfg!(target_os = "macos") && super_;
                self.input.modifiers.command = if cfg!(target_os = "macos") {
                    super_
                } else {
                    ctrl
                };

                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }

            // Things that may require repaint:
            WindowEvent::RedrawRequested
            | WindowEvent::CursorEntered { .. }
            | WindowEvent::Destroyed
            | WindowEvent::Occluded(_)
            | WindowEvent::Resized(_)
            | WindowEvent::Moved(_)
            | WindowEvent::ThemeChanged(_)
            | WindowEvent::TouchpadPressure { .. }
            | WindowEvent::CloseRequested => EventResponse {
                repaint: true,
                consumed: false,
            },

            // Things we completely ignore:
            WindowEvent::ActivationTokenDone { .. }
            | WindowEvent::AxisMotion { .. }
            | WindowEvent::DoubleTapGesture { .. }
            | WindowEvent::PanGesture { .. }
            | WindowEvent::RotationGesture { .. } => EventResponse {
                repaint: false,
                consumed: false,
            },

            WindowEvent::PinchGesture { delta, .. } => {
                // Positive delta values indicate magnification (zooming in).
                // Negative delta values indicate shrinking (zooming out).
                let zoom_factor = (*delta as f32).exp();
                self.input.events.push(egui::Event::Zoom(zoom_factor));
                EventResponse {
                    repaint: true,
                    consumed: self.ctx.wants_pointer_input(),
                }
            }
        }
    }

    /// Call this when there is a new [`accesskit::ActionRequest`].
    ///
    /// The result can be found in [`Self::egui_input`] and be extracted with [`Self::take_egui_input`].
    #[cfg(feature = "accesskit")]
    pub fn on_accesskit_action_request(&mut self, request: accesskit::ActionRequest) {
        self.egui_input
            .events
            .push(egui::Event::AccessKitActionRequest(request));
    }

    fn on_mouse_button_input
    (
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    )
    {
        if let Some(pos) = self.pointer_pos_in_points
        {
            if let Some(button) = translate_mouse_button(button)
            {
                let pressed = state == winit::event::ElementState::Pressed;

                self.input.events.push(egui::Event::PointerButton
                {
                    pos,
                    button,
                    pressed,
                    modifiers: self.input.modifiers
                });

                if self.simulate_touch_screen
                {
                    if pressed
                    {
                        self.any_pointer_button_down = true;

                        self.input.events.push(egui::Event::Touch
                        {
                            device_id: egui::TouchDeviceId(0),
                            id: egui::TouchId(0),
                            phase: egui::TouchPhase::Start,
                            pos,
                            force: None
                        })
                    } else {
                        self.any_pointer_button_down = false;

                        self.input.events.push(egui::Event::PointerGone);

                        self.input.events.push(egui::Event::Touch
                        {
                            device_id: egui::TouchDeviceId(0),
                            id: egui::TouchId(0),
                            phase: egui::TouchPhase::End,
                            pos,
                            force: None
                        })
                    }
                }
            }
        }
    }

    fn on_cursor_moved(
        &mut self,
        window: &Window,
        pos_in_pixels: winit::dpi::PhysicalPosition<f64>,
    ) {
        let pixels_per_point = pixels_per_point(&self.ctx, window);

        let pos_in_points = egui::pos2(
            pos_in_pixels.x as f32 / pixels_per_point,
            pos_in_pixels.y as f32 / pixels_per_point,
        );
        self.pointer_pos_in_points = Some(pos_in_points);

        if self.simulate_touch_screen {
            if self.any_pointer_button_down {
                self.input
                    .events
                    .push(egui::Event::PointerMoved(pos_in_points));

                self.input.events.push(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId(0),
                    phase: egui::TouchPhase::Move,
                    pos: pos_in_points,
                    force: None,
                });
            }
        } else {
            self.input
                .events
                .push(egui::Event::PointerMoved(pos_in_points));
        }
    }

    fn on_touch(&mut self, window: &Window, touch: &winit::event::Touch) {
        let pixels_per_point = pixels_per_point(&self.ctx, window);

        // Emit touch event
        self.input.events.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(egui::epaint::util::hash(touch.device_id)),
            id: egui::TouchId::from(touch.id),
            phase: match touch.phase {
                winit::event::TouchPhase::Started => egui::TouchPhase::Start,
                winit::event::TouchPhase::Moved => egui::TouchPhase::Move,
                winit::event::TouchPhase::Ended => egui::TouchPhase::End,
                winit::event::TouchPhase::Cancelled => egui::TouchPhase::Cancel,
            },
            pos: egui::pos2(
                touch.location.x as f32 / pixels_per_point,
                touch.location.y as f32 / pixels_per_point,
            ),
            force: match touch.force {
                Some(winit::event::Force::Normalized(force)) => Some(force as f32),
                Some(winit::event::Force::Calibrated {
                    force,
                    max_possible_force,
                    ..
                }) => Some((force / max_possible_force) as f32),
                None => None,
            },
        });
        // If we're not yet translating a touch or we're translating this very
        // touch …
        if self.pointer_touch_id.is_none() || self.pointer_touch_id.unwrap() == touch.id {
            // … emit PointerButton resp. PointerMoved events to emulate mouse
            match touch.phase {
                winit::event::TouchPhase::Started => {
                    self.pointer_touch_id = Some(touch.id);
                    // First move the pointer to the right location
                    self.on_cursor_moved(window, touch.location);
                    self.on_mouse_button_input(
                        winit::event::ElementState::Pressed,
                        winit::event::MouseButton::Left,
                    );
                }
                winit::event::TouchPhase::Moved => {
                    self.on_cursor_moved(window, touch.location);
                }
                winit::event::TouchPhase::Ended => {
                    self.pointer_touch_id = None;
                    self.on_mouse_button_input(
                        winit::event::ElementState::Released,
                        winit::event::MouseButton::Left,
                    );
                    // The pointer should vanish completely to not get any
                    // hover effects
                    self.pointer_pos_in_points = None;
                    self.input.events.push(egui::Event::PointerGone);
                }
                winit::event::TouchPhase::Cancelled => {
                    self.pointer_touch_id = None;
                    self.pointer_pos_in_points = None;
                    self.input.events.push(egui::Event::PointerGone);
                }
            }
        }
    }

    fn on_mouse_wheel(&mut self, window: &Window, delta: winit::event::MouseScrollDelta) {
        let pixels_per_point = pixels_per_point(&self.ctx, window);

        {
            let (unit, delta) = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    (egui::MouseWheelUnit::Line, egui::vec2(x, y))
                }
                winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                    x,
                    y,
                }) => (
                    egui::MouseWheelUnit::Point,
                    egui::vec2(x as f32, y as f32) / pixels_per_point,
                ),
            };
            let modifiers = self.input.modifiers;
            self.input.events.push(egui::Event::MouseWheel {
                unit,
                delta,
                modifiers,
            });
        }
        let delta = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                let points_per_scroll_line = 50.0; // Scroll speed decided by consensus: https://github.com/emilk/egui/issues/461
                egui::vec2(x, y) * points_per_scroll_line
            }
            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                egui::vec2(delta.x as f32, delta.y as f32) / pixels_per_point
            }
        };

        if self.input.modifiers.ctrl || self.input.modifiers.command {
            // Treat as zoom instead:
            let factor = (delta.y / 200.0).exp();
            self.input.events.push(egui::Event::Zoom(factor));
        } else if self.input.modifiers.shift {
            // Treat as horizontal scrolling.
            // Note: one Mac we already get horizontal scroll events when shift is down.
            self.input
                .events
                .push(egui::Event::MouseWheel{
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(delta.x + delta.y, 0.0),
                        modifiers: egui::Modifiers {  shift: true, ..Default::default() },
                    });
        } else {
            self.input.events.push(egui::Event::MouseWheel{
                    unit: egui::MouseWheelUnit::Point,
                    delta,
                    modifiers: Default::default(),
                });
        }
    }

    fn on_keyboard_input(&mut self, event: &winit::event::KeyEvent) -> bool {
        let winit::event::KeyEvent {
            // Represents the position of a key independent of the currently active layout.
            //
            // It also uniquely identifies the physical key (i.e. it's mostly synonymous with a scancode).
            // The most prevalent use case for this is games. For example the default keys for the player
            // to move around might be the W, A, S, and D keys on a US layout. The position of these keys
            // is more important than their label, so they should map to Z, Q, S, and D on an "AZERTY"
            // layout. (This value is `KeyCode::KeyW` for the Z key on an AZERTY layout.)
            physical_key,

            // Represents the results of a keymap, i.e. what character a certain key press represents.
            // When telling users "Press Ctrl-F to find", this is where we should
            // look for the "F" key, because they may have a dvorak layout on
            // a qwerty keyboard, and so the logical "F" character may not be located on the physical `KeyCode::KeyF` position.
            logical_key,

            text,

            state,

            location: _, // e.g. is it on the numpad?
            repeat: _,   // egui will figure this out for us
            ..
        } = event;

        let pressed = *state == winit::event::ElementState::Pressed;

        let physical_key = if let winit::keyboard::PhysicalKey::Code(keycode) = *physical_key {
            key_from_key_code(keycode)
        } else {
            None
        };

        let logical_key = key_from_winit_key(logical_key);

        if let Some(logical_key) = logical_key {

            self.input.events.push(egui::Event::Key {
                key: logical_key,
                pressed,
                repeat: false, // egui will fill this in for us!
                modifiers: self.input.modifiers,
                physical_key,
            });
        }

        if let Some(text) = &text {
            // Make sure there is text, and that it is not control characters
            // (e.g. delete is sent as "\u{f728}" on macOS).
            if !text.is_empty() && text.chars().all(is_printable_char) {
                // On some platforms we get here when the user presses Cmd-C (copy), ctrl-W, etc.
                // We need to ignore these characters that are side-effects of commands.
                // Also make sure the key is pressed (not released). On Linux, text might
                // contain some data even when the key is released.
                let is_cmd = self.input.modifiers.ctrl
                    || self.input.modifiers.command
                    || self.input.modifiers.mac_cmd;
                if pressed && !is_cmd {
                    self.input
                        .events
                        .push(egui::Event::Text(text.to_string()));
                }
            }
        }

        false
    }

    /// Call with the output given by `egui`.
    ///
    /// This will, if needed:
    /// * update the cursor
    /// * copy text to the clipboard
    /// * open any clicked urls
    /// * update the IME
    /// *
    pub fn handle_platform_output(
        &mut self,
        window: &Window,
        platform_output: egui::PlatformOutput,
    ) {

        let egui::PlatformOutput {
            cursor_icon,
            open_url,
            ..
        } = platform_output;

        self.set_cursor_icon(window, cursor_icon);

        if let Some(open_url) = open_url {
            open_url_in_browser(&open_url.url);
        }

        #[cfg(feature = "accesskit")]
        if let Some(accesskit) = self.accesskit.as_ref() {
            if let Some(update) = accesskit_update {
                accesskit.update_if_active(|| update);
            }
        }
    }

    fn set_cursor_icon(&mut self, window: &Window, cursor_icon: egui::CursorIcon) {
        if self.current_cursor_icon == Some(cursor_icon) {
            // Prevent flickering near frame boundary when Windows OS tries to control cursor icon for window resizing.
            // On other platforms: just early-out to save CPU.
            return;
        }

        let is_pointer_in_window = self.pointer_pos_in_points.is_some();
        if is_pointer_in_window {
            self.current_cursor_icon = Some(cursor_icon);

            if let Some(winit_cursor_icon) = translate_cursor(cursor_icon) {
                window.set_cursor_visible(true);
                window.set_cursor(winit_cursor_icon);
            } else {
                window.set_cursor_visible(false);
            }
        } else {
            // Remember to set the cursor again once the cursor returns to the screen:
            self.current_cursor_icon = None;
        }
    }

    /// Update the given viewport info with the current state of the window.
    ///
    /// Call before [`State::take_egui_input`].
    pub fn update_viewport_info(&mut self, window: &Window)
    {
        let pixels_per_point = pixels_per_point(&self.ctx, window);

        let viewport_info = self.input.viewports.get_mut
        (
            &self.input.viewport_id
        ).expect("attempted to access viewport info with incorect id");

        let has_a_position = match window.is_minimized() {
            None | Some(true) => false,
            Some(false) => true,
        };

        let inner_pos_px = if has_a_position {
            window
                .inner_position()
                .map(|pos| Pos2::new(pos.x as f32, pos.y as f32))
                .ok()
        } else {
            None
        };

        let outer_pos_px = if has_a_position {
            window
                .outer_position()
                .map(|pos| Pos2::new(pos.x as f32, pos.y as f32))
                .ok()
        } else {
            None
        };

        let inner_size_px = if has_a_position {
            let size = window.inner_size();
            Some(Vec2::new(size.width as f32, size.height as f32))
        } else {
            None
        };

        let outer_size_px = if has_a_position {
            let size = window.outer_size();
            Some(Vec2::new(size.width as f32, size.height as f32))
        } else {
            None
        };

        let inner_rect_px = if let (Some(pos), Some(size)) = (inner_pos_px, inner_size_px) {
            Some(Rect::from_min_size(pos, size))
        } else {
            None
        };

        let outer_rect_px = if let (Some(pos), Some(size)) = (outer_pos_px, outer_size_px) {
            Some(Rect::from_min_size(pos, size))
        } else {
            None
        };

        let inner_rect = inner_rect_px.map(|r| r / pixels_per_point);
        let outer_rect = outer_rect_px.map(|r| r / pixels_per_point);

        let monitor_size = {
            if let Some(monitor) = window.current_monitor() {
                let size = monitor.size().to_logical::<f32>(pixels_per_point.into());
                Some(egui::vec2(size.width, size.height))
            } else {
                None
            }
        };

        viewport_info.focused = Some(true); // baguette doesn't render while unfocused so this will always be true
        viewport_info.fullscreen = Some(window.fullscreen().is_some());
        viewport_info.inner_rect = inner_rect;
        viewport_info.monitor_size = monitor_size;
        viewport_info.native_pixels_per_point = Some(window.scale_factor() as f32);
        viewport_info.outer_rect = outer_rect;
        viewport_info.title = Some(window.title());

        if false {
            // It's tempting to do this, but it leads to a deadlock on Mac when running
            // `cargo run -p custom_window_frame`.
            // See https://github.com/emilk/egui/issues/3494
            viewport_info.maximized = Some(window.is_maximized());
            viewport_info.minimized = window.is_minimized().or(viewport_info.minimized);
        }
    }

    pub fn process_viewport_commands<'a>
    (
        &mut self,
        commands: impl IntoIterator<Item = &'a ViewportCommand>,
        window: &Window,
        target: &winit::event_loop::ActiveEventLoop
    )
    {
        for command in commands
        {
            self.process_viewport_command
            (
                window,
                command,
                target
            )
        }
    }
    
    fn process_viewport_command
    (
        &mut self,
        window: &Window,
        command: &ViewportCommand,
        target: &winit::event_loop::ActiveEventLoop
    )
    {
        use winit::window::ResizeDirection;
    
        let pixels_per_point = pixels_per_point(&self.ctx, window);
        
        let input = &mut self.input;
        
        let info = input.viewports.get_mut(&input.viewport_id).unwrap(); 

        match command
        {
            ViewportCommand::Close =>
            {
                target.exit();
            }
            ViewportCommand::CancelClose =>
            {
                // Need to be handled elsewhere
            }
            ViewportCommand::StartDrag =>
            {
                if let Err(err) = window.drag_window()
                {
                    match err
                    {
                        winit::error::ExternalError::NotSupported(_) => (),
                        winit::error::ExternalError::Ignored => (),
                        winit::error::ExternalError::Os(_) => (),
                    }
                }
            }
            ViewportCommand::InnerSize(size) =>
            {
                let width_px = pixels_per_point * size.x.max(1.0);
                let height_px = pixels_per_point * size.y.max(1.0);
                if window
                    .request_inner_size(PhysicalSize::new(width_px, height_px))
                    .is_some()
                {
                }
            }
            ViewportCommand::BeginResize(direction) =>
            {
                if window.drag_resize_window(match direction
                {
                    egui::ResizeDirection::North => ResizeDirection::North,
                    egui::ResizeDirection::South => ResizeDirection::South,
                    egui::ResizeDirection::East => ResizeDirection::East,
                    egui::ResizeDirection::West => ResizeDirection::West,
                    egui::ResizeDirection::NorthEast => ResizeDirection::NorthEast,
                    egui::ResizeDirection::SouthEast => ResizeDirection::SouthEast,
                    egui::ResizeDirection::NorthWest => ResizeDirection::NorthWest,
                    egui::ResizeDirection::SouthWest => ResizeDirection::SouthWest,
                }).is_err() {
                }
            }
            ViewportCommand::Title(title) => {
                window.set_title(title);
            }
            ViewportCommand::Transparent(v) => window.set_transparent(*v),
            ViewportCommand::Visible(v) => window.set_visible(*v),
            ViewportCommand::OuterPosition(pos) => {
                window.set_outer_position(PhysicalPosition::new(
                    pixels_per_point * pos.x,
                    pixels_per_point * pos.y,
                ));
            }
            ViewportCommand::MinInnerSize(s) => {
                window.set_min_inner_size((s.is_finite() && *s != Vec2::ZERO).then_some(
                    PhysicalSize::new(pixels_per_point * s.x, pixels_per_point * s.y),
                ));
            }
            ViewportCommand::MaxInnerSize(s) => {
                window.set_max_inner_size((s.is_finite() && *s != Vec2::INFINITY).then_some(
                    PhysicalSize::new(pixels_per_point * s.x, pixels_per_point * s.y),
                ));
            }
            ViewportCommand::ResizeIncrements(s) => {
                window.set_resize_increments(
                    s.map(|s| PhysicalSize::new(pixels_per_point * s.x, pixels_per_point * s.y)),
                );
            }
            ViewportCommand::Resizable(v) => window.set_resizable(*v),
            ViewportCommand::EnableButtons {
                close,
                minimized,
                maximize,
            } => window.set_enabled_buttons(
                if *close {
                    WindowButtons::CLOSE
                } else {
                    WindowButtons::empty()
                } | if *minimized {
                    WindowButtons::MINIMIZE
                } else {
                    WindowButtons::empty()
                } | if *maximize {
                    WindowButtons::MAXIMIZE
                } else {
                    WindowButtons::empty()
                },
            ),
            ViewportCommand::Minimized(v) => {
                window.set_minimized(*v);
                info.minimized = Some(*v);
            }
            ViewportCommand::Maximized(v) => {
                window.set_maximized(*v);
                info.maximized = Some(*v);
            }
            ViewportCommand::Fullscreen(v) => {
                window.set_fullscreen(v.then_some(winit::window::Fullscreen::Borderless(None)));
            }
            ViewportCommand::Decorations(v) => window.set_decorations(*v),
            ViewportCommand::WindowLevel(l) => window.set_window_level(match l {
                egui::viewport::WindowLevel::AlwaysOnBottom => WindowLevel::AlwaysOnBottom,
                egui::viewport::WindowLevel::AlwaysOnTop => WindowLevel::AlwaysOnTop,
                egui::viewport::WindowLevel::Normal => WindowLevel::Normal,
            }),
            ViewportCommand::Icon(icon) => {
                window.set_window_icon(icon.as_ref().map(|icon| {
                    winit::window::Icon::from_rgba(icon.rgba.clone(), icon.width, icon.height)
                        .expect("Invalid ICON data!")
                }));
            }
            //ViewportCommand::IMEPosition(rect) => {
            //    window.set_ime_cursor_area(
            //        PhysicalPosition::new(pixels_per_point * rect.x, pixels_per_point * rect.y),
            //        PhysicalSize::new(
            //            pixels_per_point * rect.x,
            //            pixels_per_point * rect.y,
            //        ),
            //    );
            //}
            ViewportCommand::IMEAllowed(v) => window.set_ime_allowed(*v),
            ViewportCommand::IMEPurpose(p) => window.set_ime_purpose(match p {
                egui::viewport::IMEPurpose::Password => winit::window::ImePurpose::Password,
                egui::viewport::IMEPurpose::Terminal => winit::window::ImePurpose::Terminal,
                egui::viewport::IMEPurpose::Normal => winit::window::ImePurpose::Normal,
            }),
            ViewportCommand::Focus => {
                if !window.has_focus() {
                    window.focus_window();
                }
            }
            ViewportCommand::RequestUserAttention(a) => {
                window.request_user_attention(match a {
                    egui::UserAttentionType::Reset => None,
                    egui::UserAttentionType::Critical => {
                        Some(winit::window::UserAttentionType::Critical)
                    }
                    egui::UserAttentionType::Informational => {
                        Some(winit::window::UserAttentionType::Informational)
                    }
                });
            }
            ViewportCommand::SetTheme(t) => window.set_theme(match t {
                egui::SystemTheme::Light => Some(winit::window::Theme::Light),
                egui::SystemTheme::Dark => Some(winit::window::Theme::Dark),
                egui::SystemTheme::SystemDefault => None,
            }),
            ViewportCommand::ContentProtected(v) => window.set_content_protected(*v),
            ViewportCommand::CursorPosition(pos) => {
                if window.set_cursor_position(PhysicalPosition::new(
                    pixels_per_point * pos.x,
                    pixels_per_point * pos.y,
                )).is_err() {
                }
            }
            ViewportCommand::CursorGrab(o) => {
                if window.set_cursor_grab(match o {
                    egui::viewport::CursorGrab::None => CursorGrabMode::None,
                    egui::viewport::CursorGrab::Confined => CursorGrabMode::Confined,
                    egui::viewport::CursorGrab::Locked => CursorGrabMode::Locked,
                }).is_err() {
                }
            }
            ViewportCommand::CursorVisible(v) => window.set_cursor_visible(*v),
            ViewportCommand::MousePassthrough(passthrough) => {
                if window.set_cursor_hittest(!passthrough).is_err() {
                }
            }
            ViewportCommand::Screenshot => (),
            ViewportCommand::IMERect(_) => (),
            ViewportCommand::RequestCut => (),
            ViewportCommand::RequestCopy => (),
            ViewportCommand::RequestPaste => (),
        }
    }

}

fn open_url_in_browser(_url: &str) {
    #[cfg(feature = "webbrowser")]
    if let Err(err) = webbrowser::open(_url) {
        log::warn!("Failed to open url: {}", err);
    }

    //#[cfg(not(feature = "webbrowser"))]
    //{
    //    log::warn!("Cannot open url - feature \"links\" not enabled.");
    //}
}

/// Winit sends special keys (backspace, delete, F1, …) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

//fn is_cut_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
//    keycode == egui::Key::Cut
//        || (modifiers.command && keycode == egui::Key::X)
//        || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Delete)
//}
//
//fn is_copy_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
//    keycode == egui::Key::Copy
//        || (modifiers.command && keycode == egui::Key::C)
//        || (cfg!(target_os = "windows") && modifiers.ctrl && keycode == egui::Key::Insert)
//}
//
//fn is_paste_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
//    keycode == egui::Key::Paste
//        || (modifiers.command && keycode == egui::Key::V)
//        || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Insert)
//}

/// converts from winit to egui's representation of mouse input
fn translate_mouse_button(button: winit::event::MouseButton) -> Option<egui::PointerButton> {
    match button {
        winit::event::MouseButton::Left => Some(egui::PointerButton::Primary),
        winit::event::MouseButton::Right => Some(egui::PointerButton::Secondary),
        winit::event::MouseButton::Middle => Some(egui::PointerButton::Middle),
        winit::event::MouseButton::Back => Some(egui::PointerButton::Extra1),
        winit::event::MouseButton::Forward => Some(egui::PointerButton::Extra2),
        winit::event::MouseButton::Other(_) => None,
    }
}

fn key_from_winit_key(key: &winit::keyboard::Key) -> Option<egui::Key> {
    match key {
        winit::keyboard::Key::Named(named_key) => key_from_named_key(*named_key),
        _ => None,
    }
}

fn key_from_named_key(named_key: winit::keyboard::NamedKey) -> Option<egui::Key> {
    use egui::Key;
    use winit::keyboard::NamedKey;

    Some(match named_key {
        NamedKey::Enter => Key::Enter,
        NamedKey::Tab => Key::Tab,
        NamedKey::Space => Key::Space,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::End => Key::End,
        NamedKey::Home => Key::Home,
        NamedKey::PageDown => Key::PageDown,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Delete => Key::Delete,
        NamedKey::Insert => Key::Insert,
        NamedKey::Escape => Key::Escape,
        NamedKey::F1 => Key::F1,
        NamedKey::F2 => Key::F2,
        NamedKey::F3 => Key::F3,
        NamedKey::F4 => Key::F4,
        NamedKey::F5 => Key::F5,
        NamedKey::F6 => Key::F6,
        NamedKey::F7 => Key::F7,
        NamedKey::F8 => Key::F8,
        NamedKey::F9 => Key::F9,
        NamedKey::F10 => Key::F10,
        NamedKey::F11 => Key::F11,
        NamedKey::F12 => Key::F12,
        NamedKey::F13 => Key::F13,
        NamedKey::F14 => Key::F14,
        NamedKey::F15 => Key::F15,
        NamedKey::F16 => Key::F16,
        NamedKey::F17 => Key::F17,
        NamedKey::F18 => Key::F18,
        NamedKey::F19 => Key::F19,
        NamedKey::F20 => Key::F20,
        _ => return None
    })
}

fn key_from_key_code(key: winit::keyboard::KeyCode) -> Option<egui::Key> {
    use egui::Key;
    use winit::keyboard::KeyCode;

    Some(match key {
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowUp => Key::ArrowUp,

        KeyCode::Escape => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,
        KeyCode::Space => Key::Space,

        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,

        // KeyCode::Colon => Key::Colon, // NOTE: there is no physical colon key on an american keyboard


        KeyCode::Minus | KeyCode::NumpadSubtract => Key::Minus,

        KeyCode::NumpadAdd => Key::Plus,

        KeyCode::Digit0 | KeyCode::Numpad0 => Key::Num0,
        KeyCode::Digit1 | KeyCode::Numpad1 => Key::Num1,
        KeyCode::Digit2 | KeyCode::Numpad2 => Key::Num2,
        KeyCode::Digit3 | KeyCode::Numpad3 => Key::Num3,
        KeyCode::Digit4 | KeyCode::Numpad4 => Key::Num4,
        KeyCode::Digit5 | KeyCode::Numpad5 => Key::Num5,
        KeyCode::Digit6 | KeyCode::Numpad6 => Key::Num6,
        KeyCode::Digit7 | KeyCode::Numpad7 => Key::Num7,
        KeyCode::Digit8 | KeyCode::Numpad8 => Key::Num8,
        KeyCode::Digit9 | KeyCode::Numpad9 => Key::Num9,

        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,

        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,

        _ => {
            return None;
        }
    })
}

fn translate_cursor(cursor_icon: egui::CursorIcon) -> Option<winit::window::CursorIcon> {
    match cursor_icon {
        egui::CursorIcon::None => None,

        egui::CursorIcon::Alias => Some(winit::window::CursorIcon::Alias),
        egui::CursorIcon::AllScroll => Some(winit::window::CursorIcon::AllScroll),
        egui::CursorIcon::Cell => Some(winit::window::CursorIcon::Cell),
        egui::CursorIcon::ContextMenu => Some(winit::window::CursorIcon::ContextMenu),
        egui::CursorIcon::Copy => Some(winit::window::CursorIcon::Copy),
        egui::CursorIcon::Crosshair => Some(winit::window::CursorIcon::Crosshair),
        egui::CursorIcon::Default => Some(winit::window::CursorIcon::Default),
        egui::CursorIcon::Grab => Some(winit::window::CursorIcon::Grab),
        egui::CursorIcon::Grabbing => Some(winit::window::CursorIcon::Grabbing),
        egui::CursorIcon::Help => Some(winit::window::CursorIcon::Help),
        egui::CursorIcon::Move => Some(winit::window::CursorIcon::Move),
        egui::CursorIcon::NoDrop => Some(winit::window::CursorIcon::NoDrop),
        egui::CursorIcon::NotAllowed => Some(winit::window::CursorIcon::NotAllowed),
        egui::CursorIcon::PointingHand => Some(winit::window::CursorIcon::Pointer),
        egui::CursorIcon::Progress => Some(winit::window::CursorIcon::Progress),

        egui::CursorIcon::ResizeHorizontal => Some(winit::window::CursorIcon::EwResize),
        egui::CursorIcon::ResizeNeSw => Some(winit::window::CursorIcon::NeswResize),
        egui::CursorIcon::ResizeNwSe => Some(winit::window::CursorIcon::NwseResize),
        egui::CursorIcon::ResizeVertical => Some(winit::window::CursorIcon::NsResize),

        egui::CursorIcon::ResizeEast => Some(winit::window::CursorIcon::EResize),
        egui::CursorIcon::ResizeSouthEast => Some(winit::window::CursorIcon::SeResize),
        egui::CursorIcon::ResizeSouth => Some(winit::window::CursorIcon::SResize),
        egui::CursorIcon::ResizeSouthWest => Some(winit::window::CursorIcon::SwResize),
        egui::CursorIcon::ResizeWest => Some(winit::window::CursorIcon::WResize),
        egui::CursorIcon::ResizeNorthWest => Some(winit::window::CursorIcon::NwResize),
        egui::CursorIcon::ResizeNorth => Some(winit::window::CursorIcon::NResize),
        egui::CursorIcon::ResizeNorthEast => Some(winit::window::CursorIcon::NeResize),
        egui::CursorIcon::ResizeColumn => Some(winit::window::CursorIcon::ColResize),
        egui::CursorIcon::ResizeRow => Some(winit::window::CursorIcon::RowResize),

        egui::CursorIcon::Text => Some(winit::window::CursorIcon::Text),
        egui::CursorIcon::VerticalText => Some(winit::window::CursorIcon::VerticalText),
        egui::CursorIcon::Wait => Some(winit::window::CursorIcon::Wait),
        egui::CursorIcon::ZoomIn => Some(winit::window::CursorIcon::ZoomIn),
        egui::CursorIcon::ZoomOut => Some(winit::window::CursorIcon::ZoomOut),
    }
}

// Helpers for egui Viewports
// ---------------------------------------------------------------------------

/// Short and fast description of an event.
/// Useful for logging and profiling.
pub fn short_generic_event_description<T>(event: &winit::event::Event<T>) -> &'static str {
    use winit::event::{DeviceEvent, Event, StartCause};

    match event {
        Event::AboutToWait => "Event::AboutToWait",
        Event::LoopExiting => "Event::LoopExiting",
        Event::Suspended => "Event::Suspended",
        Event::Resumed => "Event::Resumed",
        Event::MemoryWarning => "Event::MemoryWarning",
        Event::UserEvent(_) => "UserEvent",
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::Added { .. } => "DeviceEvent::Added",
            DeviceEvent::Removed { .. } => "DeviceEvent::Removed",
            DeviceEvent::MouseMotion { .. } => "DeviceEvent::MouseMotion",
            DeviceEvent::MouseWheel { .. } => "DeviceEvent::MouseWheel",
            DeviceEvent::Motion { .. } => "DeviceEvent::Motion",
            DeviceEvent::Button { .. } => "DeviceEvent::Button",
            DeviceEvent::Key { .. } => "DeviceEvent::Key",
        },
        Event::NewEvents(start_cause) => match start_cause {
            StartCause::ResumeTimeReached { .. } => "NewEvents::ResumeTimeReached",
            StartCause::WaitCancelled { .. } => "NewEvents::WaitCancelled",
            StartCause::Poll => "NewEvents::Poll",
            StartCause::Init => "NewEvents::Init",
        },
        Event::WindowEvent { event, .. } => short_window_event_description(event),
    }
}

/// Short and fast description of an event.
/// Useful for logging and profiling.
pub fn short_window_event_description(event: &winit::event::WindowEvent) -> &'static str {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::ActivationTokenDone { .. } => "WindowEvent::ActivationTokenDone",
        WindowEvent::Resized { .. } => "WindowEvent::Resized",
        WindowEvent::Moved { .. } => "WindowEvent::Moved",
        WindowEvent::CloseRequested { .. } => "WindowEvent::CloseRequested",
        WindowEvent::Destroyed { .. } => "WindowEvent::Destroyed",
        WindowEvent::DroppedFile { .. } => "WindowEvent::DroppedFile",
        WindowEvent::HoveredFile { .. } => "WindowEvent::HoveredFile",
        WindowEvent::HoveredFileCancelled { .. } => "WindowEvent::HoveredFileCancelled",
        WindowEvent::Focused { .. } => "WindowEvent::Focused",
        WindowEvent::KeyboardInput { .. } => "WindowEvent::KeyboardInput",
        WindowEvent::ModifiersChanged { .. } => "WindowEvent::ModifiersChanged",
        WindowEvent::Ime { .. } => "WindowEvent::Ime",
        WindowEvent::CursorMoved { .. } => "WindowEvent::CursorMoved",
        WindowEvent::CursorEntered { .. } => "WindowEvent::CursorEntered",
        WindowEvent::CursorLeft { .. } => "WindowEvent::CursorLeft",
        WindowEvent::MouseWheel { .. } => "WindowEvent::MouseWheel",
        WindowEvent::MouseInput { .. } => "WindowEvent::MouseInput",
        WindowEvent::RedrawRequested { .. } => "WindowEvent::RedrawRequested",
        WindowEvent::TouchpadPressure { .. } => "WindowEvent::TouchpadPressure",
        WindowEvent::AxisMotion { .. } => "WindowEvent::AxisMotion",
        WindowEvent::Touch { .. } => "WindowEvent::Touch",
        WindowEvent::ScaleFactorChanged { .. } => "WindowEvent::ScaleFactorChanged",
        WindowEvent::ThemeChanged { .. } => "WindowEvent::ThemeChanged",
        WindowEvent::Occluded { .. } => "WindowEvent::Occluded",
        WindowEvent::PinchGesture { .. } =>"WindowEvent::PinchGesture",
        WindowEvent::PanGesture { .. } => "WindowEvent::PanGesture",
        WindowEvent::DoubleTapGesture { .. } => "WindowEvent::DoubleTapGesture",
        WindowEvent::RotationGesture { .. } => "WindowEvent::RotationGesture",
    }
}