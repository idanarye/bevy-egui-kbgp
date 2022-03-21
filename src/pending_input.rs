use crate::egui;
use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::{KbgpCommon, KbgpInput};

/// Handle for
/// [`kbgp_pending_input_manual`](crate::KbgpEguiResponseExt::kbgp_pending_input_manual).
pub struct KbgpInputManualHandle<'a> {
    pub(crate) state: &'a mut KbgpPendingInputState,
}

impl<'a> KbgpInputManualHandle<'a> {
    /// All the keys and buttons currently pressed during this frame.
    ///
    /// Does not include the keys/buttons presses that activated the input setting.
    pub fn input_this_frame(&'a self) -> impl 'a + Iterator<Item = KbgpInput> {
        self.state.input_this_frame.iter().cloned()
    }

    /// The input already received so far.
    pub fn received_input(&self) -> &HashSet<KbgpInput> {
        &self.state.received_input
    }

    /// Add input from `input_this_frame` to `received_input`.
    ///
    /// * `should_add` can be used to decided which input to add based on the nature of that new
    ///   input and on already existing input.
    /// * When adding a positive gamepad axis, if the negative input of the same axis was
    ///   previously added it will be removed - and vice versa.
    pub fn process_new_input(&mut self, mut should_add: impl FnMut(&Self, KbgpInput) -> bool) {
        for input in self.state.input_this_frame.iter() {
            if should_add(self, input.clone()) {
                match input {
                    KbgpInput::GamepadAxisPositive(gamepad_axis) => {
                        self.state
                            .received_input
                            .remove(&KbgpInput::GamepadAxisNegative(*gamepad_axis));
                    }
                    KbgpInput::GamepadAxisNegative(gamepad_axis) => {
                        self.state
                            .received_input
                            .remove(&KbgpInput::GamepadAxisPositive(*gamepad_axis));
                    }
                    _ => {}
                }
                self.state.received_input.insert(input.clone());
            }
        }
    }

    /// Format A string representing the currently received input.
    pub fn format_current_chord(&self) -> String {
        KbgpInput::format_chord(self.received_input().iter().cloned())
    }

    /// Show a tooltip of the currently received input.
    pub fn show_current_chord(&self, response: &egui::Response) {
        egui::containers::popup::show_tooltip_for(
            &response.ctx,
            egui::Id::null(),
            &response.rect,
            |ui| {
                ui.label(&self.format_current_chord());
            },
        );
    }
}

pub(crate) struct KbgpPendingInputState {
    pub(crate) acceptor_id: egui::Id,
    input_this_frame: Vec<KbgpInput>,
    ignored_input: Option<HashSet<KbgpInput>>,
    received_input: HashSet<KbgpInput>,
}

impl KbgpPendingInputState {
    pub(crate) fn new(acceptor_id: egui::Id) -> Self {
        Self {
            acceptor_id,
            input_this_frame: Default::default(),
            ignored_input: None,
            received_input: Default::default(),
        }
    }

    pub(crate) fn prepare(
        &mut self,
        _common: &KbgpCommon,
        _egui_ctx: &egui::Context,
        prepare_dlg: impl FnOnce(&mut KbgpPreparePendingInput),
    ) {
        let mut handle = KbgpPreparePendingInput {
            current_input: Vec::new(),
        };
        prepare_dlg(&mut handle);
        if let Some(ignored_input) = self.ignored_input.as_mut() {
            ignored_input.retain(|input| handle.current_input.contains(input));
            self.input_this_frame = handle
                .current_input
                .iter()
                .filter(|inp| !ignored_input.contains(inp))
                .cloned()
                .collect();
        } else {
            self.ignored_input = Some(handle.current_input.iter().cloned().collect());
        }
    }
}

/// An option of [`KbgpPrepare`](crate::KbgpPrepare).
pub struct KbgpPreparePendingInput {
    current_input: Vec<KbgpInput>,
}

impl KbgpPreparePendingInput {
    /// Notify KBGP about a single input was accepted from the player.
    pub fn accept_input(&mut self, input: KbgpInput) {
        self.current_input.push(input);
    }

    /// Notify KBGP about a single input was accepted from the player, only if the same input was
    /// not already received this frame.
    pub fn accept_input_unique(&mut self, input: KbgpInput) {
        if !self.current_input.contains(&input) {
            self.current_input.push(input);
        }
    }

    /// Notify KBGP about multiple inputs accepted from the player.
    pub fn accept_inputs(&mut self, inputs: impl Iterator<Item = KbgpInput>) {
        self.current_input.extend(inputs);
    }

    /// Notify KBGP about all the input from the keyboard.
    pub fn accept_keyboard_input(&mut self, keys: &Input<KeyCode>) {
        self.accept_inputs(keys.get_pressed().copied().map(KbgpInput::Keyboard));
    }

    /// Notify KBGP about all the input from mouse buttons.
    pub fn accept_mouse_buttons_input(&mut self, buttons: &Input<MouseButton>) {
        self.accept_inputs(buttons.get_pressed().copied().map(KbgpInput::MouseButton));
    }

    /// Notify KBGP about all the input from the mouse wheel.
    pub fn accept_mouse_wheel_event(
        &mut self,
        event: &bevy::input::mouse::MouseWheel,
        vertical: bool,
        horizontal: bool,
    ) {
        if vertical {
            if 0.0 < event.y {
                self.accept_input_unique(KbgpInput::MouseWheelUp);
            } else if event.y < 0.0 {
                self.accept_input_unique(KbgpInput::MouseWheelDown);
            }
        }
        if horizontal {
            if 0.0 < event.x {
                self.accept_input_unique(KbgpInput::MouseWheelRight);
            } else if event.x < 0.0 {
                self.accept_input_unique(KbgpInput::MouseWheelLeft);
            }
        }
    }

    /// Notify KBGP about all the input from the gamepad.
    pub fn accept_gamepad_input(
        &mut self,
        gamepads: &Gamepads,
        axes: &Axis<GamepadAxis>,
        buttons: &Input<GamepadButton>,
    ) {
        self.accept_inputs(buttons.get_pressed().copied().map(KbgpInput::GamepadButton));
        for gamepad in gamepads.iter() {
            for gamepad_axis_type in [
                GamepadAxisType::LeftStickX,
                GamepadAxisType::LeftStickY,
                GamepadAxisType::LeftZ,
                GamepadAxisType::RightStickX,
                GamepadAxisType::RightStickY,
                GamepadAxisType::RightZ,
                GamepadAxisType::DPadX,
                GamepadAxisType::DPadY,
            ] {
                let gamepad_axis = GamepadAxis(*gamepad, gamepad_axis_type);
                if let Some(axis_value) = axes.get(gamepad_axis) {
                    if 0.5 < axis_value {
                        self.accept_input(KbgpInput::GamepadAxisPositive(gamepad_axis));
                    } else if axis_value < -0.5 {
                        self.accept_input(KbgpInput::GamepadAxisNegative(gamepad_axis));
                    }
                }
            }
        }
        let _ = (gamepads, axes);
    }
}
