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
                if !ignored_input.contains(input) {
                    self.received_input.insert(input.clone());
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
    pub fn accept_keyboard_input(&mut self, keys: &Input<KeyCode>) {
        self.current_input
            .extend(keys.get_pressed().copied().map(KbgpInput::Keyboard));
    }

    pub fn accept_gamepad_input(
        &mut self,
        gamepads: &Gamepads,
        axes: &Axis<GamepadAxis>,
        buttons: &Input<GamepadButton>,
    ) {
        let _ = (gamepads, axes, buttons);
    }
}
