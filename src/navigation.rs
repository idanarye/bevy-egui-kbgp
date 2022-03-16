use bevy::prelude::*;

use crate::{kbgp_get, KbgpCommon, KbgpPrepareHandle, KbgpState, NodeData};

const INPUT_MASK_UP: u8 = 1;
const INPUT_MASK_DOWN: u8 = 2;
const INPUT_MASK_VERTICAL: u8 = INPUT_MASK_UP | INPUT_MASK_DOWN;
const INPUT_MASK_LEFT: u8 = 4;
const INPUT_MASK_RIGHT: u8 = 8;
const INPUT_MASK_HORIZONTAL: u8 = INPUT_MASK_LEFT | INPUT_MASK_RIGHT;

const INPUT_MASK_ACTIVATE: u8 = 16;

#[derive(Default)]
pub(crate) struct KbgpNavigationState {
    move_focus: Option<egui::Id>,
    activate: Option<egui::Id>,
    prev_input: u8,
    next_navigation: f64,
}

impl KbgpPrepareHandle {
    /// Move the focus one widget up. If no widget has the focus - move up from the bottom.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    pub fn navigate_up(&mut self) {
        self.input |= INPUT_MASK_UP;
    }

    /// Move the focus one widget down. If no widget has the focus - move down from the top.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    pub fn navigate_down(&mut self) {
        self.input |= INPUT_MASK_DOWN;
    }

    /// Move the focus one widget left. If no widget has the focus - move left from the right.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    pub fn navigate_left(&mut self) {
        self.input |= INPUT_MASK_LEFT;
    }

    /// Move the focus one widget right. If no widget has the focus - move right from the left.
    ///
    /// Will only work if [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) was
    /// called on the currently focused widget, and can only target widgets marked
    /// `kbgp_navigation` was called on.
    pub fn navigate_right(&mut self) {
        self.input |= INPUT_MASK_RIGHT;
    }

    /// Activate the currently focused widget.
    ///
    /// Will only work if the widget's activation is checked with
    /// [`kbgp_activated`](crate::KbgpEguiResponseExt::kbgp_activated). Cannot affect egui's
    /// standard `clicked`.
    pub fn activate_focused(&mut self) {
        self.input |= INPUT_MASK_ACTIVATE;
    }

    /// Navigate the UI with arrow keys.
    pub fn navigate_keyboard_default(&mut self, keys: &Input<KeyCode>) {
        for key in keys.get_pressed() {
            match key {
                KeyCode::Up => self.navigate_up(),
                KeyCode::Down => self.navigate_down(),
                KeyCode::Left => self.navigate_left(),
                KeyCode::Right => self.navigate_right(),
                _ => (),
            }
        }
    }

    /// Navigate the UI with gamepads.
    ///
    /// * Use both left stick and d-pad for navigation.
    /// * Use both the south button and the start button for activation.
    pub fn navigate_gamepad_default(
        &mut self,
        gamepads: &Gamepads,
        axes: &Axis<GamepadAxis>,
        buttons: &Input<GamepadButton>,
    ) {
        for gamepad in gamepads.iter() {
            for (axis_type, mask_for_negative, mask_for_positive) in [
                (GamepadAxisType::DPadX, INPUT_MASK_LEFT, INPUT_MASK_RIGHT),
                (GamepadAxisType::DPadY, INPUT_MASK_DOWN, INPUT_MASK_UP),
                (
                    GamepadAxisType::LeftStickX,
                    INPUT_MASK_LEFT,
                    INPUT_MASK_RIGHT,
                ),
                (GamepadAxisType::LeftStickY, INPUT_MASK_DOWN, INPUT_MASK_UP),
            ] {
                if let Some(axis_value) = axes.get(GamepadAxis(*gamepad, axis_type)) {
                    if axis_value < -0.5 {
                        self.input |= mask_for_negative;
                    } else if 0.5 < axis_value {
                        self.input |= mask_for_positive;
                    }
                }
            }
        }
        for GamepadButton(_, button_type) in buttons.get_pressed() {
            match button_type {
                GamepadButtonType::DPadUp => self.navigate_up(),
                GamepadButtonType::DPadDown => self.navigate_down(),
                GamepadButtonType::DPadLeft => self.navigate_left(),
                GamepadButtonType::DPadRight => self.navigate_right(),
                GamepadButtonType::South | GamepadButtonType::Start => {
                    self.activate_focused();
                }
                _ => (),
            }
        }
    }
}

impl KbgpNavigationState {
    pub(crate) fn prepare(
        &mut self,
        common: &KbgpCommon,
        egui_ctx: &egui::CtxRef,
        prepare_dlg: impl FnOnce(&mut KbgpPrepareHandle),
    ) {
        self.move_focus = None;
        self.activate = None;

        let mut handle = KbgpPrepareHandle {
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

            if effective_input & INPUT_MASK_ACTIVATE != 0 {
                self.activate = egui_ctx.memory().focus();
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
}

impl KbgpNavigationState {
    fn move_focus(
        &mut self,
        common: &KbgpCommon,
        egui_ctx: &egui::CtxRef,
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

/// Extensions for egui's `Response` to activate KBGP's functionality.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_egui_kbgp::prelude::*;
/// # let ui: egui::Ui = todo!();
/// if ui
///     .button("My Button")
///     .kbgp_initial_focus() // focus on this button when starting the UI
///     .kbgp_navigation() // navigate to and from this button with keyboard/gamepad
///     .kbgp_activated() // use instead of egui's `.clicked()` to support gamepads
/// {
///     // ...
/// }
/// ```
pub trait KbgpEguiResponseExt {
    /// When the UI is first created, focus on this widget.
    ///
    /// Must be called before [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation).
    fn kbgp_initial_focus(self) -> Self;
    /// Navigate to and from this widget.
    fn kbgp_navigation(self) -> Self;
    /// Use instead of egui's `.clicked()` to support gamepads.
    fn kbgp_activated(self) -> bool;
}

impl KbgpEguiResponseExt for egui::Response {
    fn kbgp_initial_focus(self) -> Self {
        let kbgp = kbgp_get(&self.ctx);
        let kbgp = kbgp.lock();
        match kbgp.state {
            KbgpState::Inactive => {
                self.request_focus();
            }
            KbgpState::Navigation(_) => {}
        }
        self
    }

    fn kbgp_navigation(self) -> Self {
        let kbgp = kbgp_get(&self.ctx);
        let mut kbgp = kbgp.lock();
        kbgp.common.nodes.insert(
            self.id,
            NodeData {
                rect: self.rect,
                stale_counter: 0,
            },
        );
        match &kbgp.state {
            KbgpState::Inactive => {}
            KbgpState::Navigation(state) => {
                if Some(self.id) == state.move_focus || self.clicked() {
                    self.request_focus();
                }
            }
        }
        self
    }

    fn kbgp_activated(self) -> bool {
        let kbgp = kbgp_get(&self.ctx);
        let kbgp = kbgp.lock();
        match &kbgp.state {
            KbgpState::Inactive => self.clicked(),
            KbgpState::Navigation(state) => self.clicked() || Some(self.id) == state.activate,
        }
    }
}
