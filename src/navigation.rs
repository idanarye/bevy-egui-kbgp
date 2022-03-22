use crate::egui;
use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::KbgpCommon;

const INPUT_MASK_UP: u8 = 1;
const INPUT_MASK_DOWN: u8 = 2;
const INPUT_MASK_VERTICAL: u8 = INPUT_MASK_UP | INPUT_MASK_DOWN;
const INPUT_MASK_LEFT: u8 = 4;
const INPUT_MASK_RIGHT: u8 = 8;
const INPUT_MASK_HORIZONTAL: u8 = INPUT_MASK_LEFT | INPUT_MASK_RIGHT;

const INPUT_MASK_CLICK: u8 = 16;

#[derive(Default)]
pub(crate) struct KbgpNavigationState {
    pub(crate) move_focus: Option<egui::Id>,
    prev_input: u8,
    next_navigation: f64,
}

/// An option of [`KbgpPrepare`](crate::KbgpPrepare).
pub struct KbgpPrepareNavigation {
    /// When the player holds a key/button, KBGP will wait `secs_after_first_input` seconds before
    /// starting to rapidly apply the action.
    ///
    /// Default: 0.6 seconds.
    pub secs_after_first_input: f64,
    /// When the player holds a key/button, after
    /// [`secs_after_first_input`](crate::KbgpPrepareNavigation::secs_after_first_input), KBGP
    /// will apply the action every `secs_between_inputs` seconds.
    ///
    /// Default: 0.04 seconds.
    pub secs_between_inputs: f64,
    pub(crate) input: u8,
}

impl KbgpPrepareNavigation {
    pub fn apply_action(&mut self, action: KbgpNavAction) {
        self.input |= match action {
            KbgpNavAction::NavigateUp => INPUT_MASK_UP,
            KbgpNavAction::NavigateDown => INPUT_MASK_DOWN,
            KbgpNavAction::NavigateLeft => INPUT_MASK_LEFT,
            KbgpNavAction::NavigateRight => INPUT_MASK_RIGHT,
            KbgpNavAction::Activate(egui::PointerButton::Primary) => INPUT_MASK_CLICK,
            KbgpNavAction::Activate(egui::PointerButton::Secondary) => todo!(),
            KbgpNavAction::Activate(egui::PointerButton::Middle) => todo!(),
            KbgpNavAction::Cancel => todo!(),
        };
    }

    /// Navigate the UI with the keyboard.
    pub fn navigate_keyboard_by_binding(&mut self, keys: &Input<KeyCode>, binding: &HashMap<KeyCode, KbgpNavAction>) {
        for key in keys.get_pressed() {
            if let Some(action) = binding.get(key) {
                self.apply_action(*action);
            }
        }
    }

    /// Navigate the UI with gamepads.
    ///
    /// * Use both left stick and d-pad for navigation.
    pub fn navigate_gamepad_by_binding(
        &mut self,
        gamepads: &Gamepads,
        axes: &Axis<GamepadAxis>,
        buttons: &Input<GamepadButton>,
        binding: &HashMap<GamepadButtonType, KbgpNavAction>,
    ) {
        for gamepad in gamepads.iter() {
            for (axis_type, action_for_negative, action_for_positive) in [
                (GamepadAxisType::DPadX, KbgpNavAction::NavigateLeft, KbgpNavAction::NavigateRight),
                (GamepadAxisType::DPadY, KbgpNavAction::NavigateDown, KbgpNavAction::NavigateUp),
                (
                    GamepadAxisType::LeftStickX,
                    KbgpNavAction::NavigateLeft,
                    KbgpNavAction::NavigateRight,
                ),
                (GamepadAxisType::LeftStickY, KbgpNavAction::NavigateDown, KbgpNavAction::NavigateUp),
            ] {
                if let Some(axis_value) = axes.get(GamepadAxis(*gamepad, axis_type)) {
                    if axis_value < -0.5 {
                        self.apply_action(action_for_negative)
                    } else if 0.5 < axis_value {
                        self.apply_action(action_for_positive)
                    }
                }
            }
        }
        for GamepadButton(_, button_type) in buttons.get_pressed() {
            if let Some(action) = binding.get(button_type) {
                self.apply_action(*action);
            }
        }
    }
}

impl KbgpNavigationState {
    pub(crate) fn prepare(
        &mut self,
        common: &KbgpCommon,
        egui_ctx: &egui::Context,
        prepare_dlg: impl FnOnce(&mut KbgpPrepareNavigation),
    ) {
        self.move_focus = None;

        let mut handle = KbgpPrepareNavigation {
            secs_after_first_input: 0.6,
            secs_between_inputs: 0.04,
            input: 0,
        };

        prepare_dlg(&mut handle);
        if handle.input != 0 {
            let mut effective_input = handle.input;
            let current_time = egui_ctx.input().time;
            if self.prev_input != handle.input {
                effective_input &= !self.prev_input;
                self.next_navigation = current_time + handle.secs_after_first_input;
            } else if current_time < self.next_navigation {
                effective_input = 0;
            } else {
                self.next_navigation = current_time + handle.secs_between_inputs;
            }

            if effective_input & INPUT_MASK_CLICK != 0 {
                egui_ctx.input_mut().events.push(egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    modifiers: Default::default(),
                });
            }

            match effective_input & INPUT_MASK_VERTICAL {
                INPUT_MASK_UP => {
                    self.move_focus(common, egui_ctx, |egui::Pos2 { x, y }| egui::Pos2 {
                        x: -x,
                        y: -y,
                    });
                }
                INPUT_MASK_DOWN => {
                    self.move_focus(common, egui_ctx, |p| p);
                }
                _ => {}
            }
            // Note: Doing transpose instead of rotation so that starting navigation without
            // anything focused will make left similar to up and right similar to down.
            match effective_input & INPUT_MASK_HORIZONTAL {
                INPUT_MASK_LEFT => {
                    self.move_focus(common, egui_ctx, |egui::Pos2 { x, y }| egui::Pos2 {
                        x: -y,
                        y: -x,
                    });
                }
                INPUT_MASK_RIGHT => {
                    self.move_focus(common, egui_ctx, |egui::Pos2 { x, y }| egui::Pos2 {
                        x: y,
                        y: x,
                    });
                }
                _ => {}
            }
        }
        self.prev_input = handle.input;
    }

    fn move_focus(
        &mut self,
        common: &KbgpCommon,
        egui_ctx: &egui::Context,
        transform_pos_downward: impl Fn(egui::Pos2) -> egui::Pos2,
    ) {
        let transform_rect_downward = |rect: egui::Rect| -> egui::Rect {
            let egui::Pos2 {
                x: mut left,
                y: mut top,
            } = transform_pos_downward(rect.min);
            let egui::Pos2 {
                x: mut right,
                y: mut bottom,
            } = transform_pos_downward(rect.max);
            if right < left {
                std::mem::swap(&mut left, &mut right);
            }
            if bottom < top {
                std::mem::swap(&mut top, &mut bottom);
            }
            egui::Rect {
                min: egui::Pos2 { x: left, y: top },
                max: egui::Pos2 {
                    x: right,
                    y: bottom,
                },
            }
        };
        let transformed_nodes = common
            .nodes
            .iter()
            .map(|(id, data)| (id, transform_rect_downward(data.rect)));
        let focused_node_id = egui_ctx.memory().focus();
        let move_to = if let Some(focused_node_id) = focused_node_id {
            let focused_node_rect = if let Some(data) = common.nodes.get(&focused_node_id) {
                transform_rect_downward(data.rect)
            } else {
                return;
            };

            #[derive(Debug)]
            struct InfoForComparison {
                min_y: f32,
                max_y: f32,
                x_drift: f32,
            }
            transformed_nodes
                .filter_map(|(id, rect)| {
                    if *id == focused_node_id {
                        return None;
                    }
                    let min_y_diff = rect.min.y - focused_node_rect.max.y;
                    if min_y_diff < 0.0 {
                        return None;
                    }
                    Some((
                        id,
                        InfoForComparison {
                            min_y: min_y_diff,
                            max_y: rect.max.y - focused_node_rect.max.y,
                            x_drift: {
                                if focused_node_rect.max.x < rect.min.x {
                                    rect.max.x - focused_node_rect.min.x
                                } else if rect.max.x < focused_node_rect.min.x {
                                    focused_node_rect.max.x - rect.min.x
                                } else {
                                    0.0
                                }
                            },
                        },
                    ))
                })
                .min_by(|(_, a), (_, b)| {
                    if a.max_y < b.min_y && b.max_y < a.min_y {
                        a.x_drift.partial_cmp(&b.x_drift).unwrap()
                    } else {
                        (a.min_y + a.x_drift)
                            .partial_cmp(&(b.min_y + b.x_drift))
                            .unwrap()
                    }
                })
                .map(|(id, _)| id)
        } else {
            transformed_nodes
                .map(|(id, rect)| (id, (rect.min.y, rect.min.x)))
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(id, _)| id)
        };
        if let Some(id) = move_to {
            self.move_focus = Some(*id);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KbgpNavAction {
    /// Move the focus one widget up. If no widget has the focus - move up from the bottom.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    NavigateUp,
    /// Move the focus one widget down. If no widget has the focus - move down from the top.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    NavigateDown,
    /// Move the focus one widget left. If no widget has the focus - move left from the right.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    NavigateLeft,
    /// Move the focus one widget right. If no widget has the focus - move right from the left.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    NavigateRight,
    /// Make egui think the player clicked on the focused widget.
    Activate(egui::PointerButton),
    Cancel,
}

pub struct KbgpNavBindings {
    pub keyboard: HashMap<KeyCode, KbgpNavAction>,
    pub gamepad_buttons: HashMap<GamepadButtonType, KbgpNavAction>,
}

impl Default for KbgpNavBindings {
    fn default() -> Self {
        Self::empty()
            // Keyboard binding. No need for Space and Enter - egui does them by default.
            .with_key(KeyCode::Up, KbgpNavAction::NavigateUp)
            .with_key(KeyCode::Down, KbgpNavAction::NavigateDown)
            .with_key(KeyCode::Left, KbgpNavAction::NavigateLeft)
            .with_key(KeyCode::Right, KbgpNavAction::NavigateRight)
            // .with_key(KeyCode::Escape, KbgpNavAction::Cancel)
            // Gamepad bindings. Axis type bindings are not configurable here.
            .with_gamepad_button(GamepadButtonType::DPadUp, KbgpNavAction::NavigateUp)
            .with_gamepad_button(GamepadButtonType::DPadDown, KbgpNavAction::NavigateDown)
            .with_gamepad_button(GamepadButtonType::DPadLeft, KbgpNavAction::NavigateLeft)
            .with_gamepad_button(GamepadButtonType::DPadRight, KbgpNavAction::NavigateRight)
            .with_gamepad_button(GamepadButtonType::South, KbgpNavAction::Activate(egui::PointerButton::Primary))
            .with_gamepad_button(GamepadButtonType::East, KbgpNavAction::Cancel)
    }
}

impl KbgpNavBindings {
    pub fn empty() -> Self {
        Self {
            keyboard: Default::default(),
            gamepad_buttons: Default::default(),
        }
    }

    pub fn bind_key(&mut self, key: KeyCode, action: KbgpNavAction) {
        self.keyboard.insert(key, action);
    }

    pub fn with_key(mut self, key: KeyCode, action: KbgpNavAction) -> Self {
        self.bind_key(key, action);
        self
    }

    pub fn bind_gamepad_button(&mut self, gamepad_button: GamepadButtonType, action: KbgpNavAction) {
        self.gamepad_buttons.insert(gamepad_button, action);
    }

    pub fn with_gamepad_button(mut self, gamepad_button: GamepadButtonType, action: KbgpNavAction) -> Self {
        self.bind_gamepad_button(gamepad_button, action);
        self
    }
}
