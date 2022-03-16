use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::{KbgpCommon, KbgpInput};

pub(crate) struct KbgpPendingInputState {
    pub(crate) acceptor_id: egui::Id,
    pub(crate) ignored_input: Option<HashSet<KbgpInput>>,
    pub(crate) received_input: HashSet<KbgpInput>,
    pub(crate) finished: bool,
    pub(crate) limit: usize,
}

impl KbgpPendingInputState {
    pub(crate) fn prepare(
        &mut self,
        _common: &KbgpCommon,
        _egui_ctx: &egui::CtxRef,
        prepare_dlg: impl FnOnce(&mut KbgpPreparePendingInput),
    ) {
        let mut handle = KbgpPreparePendingInput {
            current_input: Vec::new(),
        };
        prepare_dlg(&mut handle);
        if let Some(ignored_input) = self.ignored_input.as_mut() {
            for input in handle.current_input.iter() {
                if self.limit <= self.received_input.len() {
                    break;
                }
                if ignored_input.contains(input) {
                    continue;
                }
                self.received_input.insert(input.clone());
                match input {
                    KbgpInput::GamepadAxisPositive(gamepad_axis) => {
                        self.received_input
                            .remove(&KbgpInput::GamepadAxisNegative(*gamepad_axis));
                    }
                    KbgpInput::GamepadAxisNegative(gamepad_axis) => {
                        self.received_input
                            .remove(&KbgpInput::GamepadAxisPositive(*gamepad_axis));
                    }
                    _ => {}
                }
            }
            ignored_input.retain(|input| handle.current_input.contains(input));
            if !self.received_input.is_empty() {
                if handle.current_input.is_empty() {
                    self.finished = true;
                } else if self.limit <= self.received_input.len() && ignored_input.is_empty() {
                    if !handle
                        .current_input
                        .iter()
                        .any(|input| self.received_input.contains(input))
                    {
                        self.finished = true;
                    }
                }
            }
        } else {
            self.ignored_input = Some(handle.current_input.iter().cloned().collect());
        }
    }
}

pub struct KbgpPreparePendingInput {
    current_input: Vec<KbgpInput>,
}

impl KbgpPreparePendingInput {
    pub fn accept_input(&mut self, input: KbgpInput) {
        self.current_input.push(input);
    }

    pub fn accept_inputs(&mut self, inputs: impl Iterator<Item = KbgpInput>) {
        self.current_input.extend(inputs);
    }

    pub fn accept_keyboard_input(&mut self, keys: &Input<KeyCode>) {
        self.accept_inputs(keys.get_pressed().copied().map(KbgpInput::Keyboard));
    }

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
