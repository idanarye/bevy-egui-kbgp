use bevy::prelude::*;
use bevy::utils::HashMap;

const INPUT_MASK_UP: u8 = 1;
const INPUT_MASK_DOWN: u8 = 2;
const INPUT_MASK_VERTICAL: u8 = INPUT_MASK_UP | INPUT_MASK_DOWN;
const INPUT_MASK_LEFT: u8 = 4;
const INPUT_MASK_RIGHT: u8 = 8;
const INPUT_MASK_HORIZONTAL: u8 = INPUT_MASK_LEFT | INPUT_MASK_RIGHT;

const INPUT_MASK_ACTIVATE: u8 = 16;

pub struct KbgpPrepareHandle {
    pub secs_after_first_movement: f64,
    pub secs_between_movements: f64,
    input: u8,
}

impl KbgpPrepareHandle {
    pub fn navigate_up(&mut self) {
        self.input |= INPUT_MASK_UP;
    }

    pub fn navigate_down(&mut self) {
        self.input |= INPUT_MASK_DOWN;
    }

    pub fn navigate_left(&mut self) {
        self.input |= INPUT_MASK_LEFT;
    }

    pub fn navigate_right(&mut self) {
        self.input |= INPUT_MASK_RIGHT;
    }

    pub fn activate_focused(&mut self) {
        self.input |= INPUT_MASK_ACTIVATE;
    }

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

#[derive(Debug)]
struct NodeData {
    rect: egui::Rect,
    still_there: bool,
}

#[derive(Default)]
pub struct Kbgp {
    nodes: HashMap<egui::Id, NodeData>,
    move_focus: Option<egui::Id>,
    activate: Option<egui::Id>,
    prev_input: u8,
    next_navigation: f64,
}

impl Kbgp {
    pub fn prepare(
        &mut self,
        egui_ctx: &egui::CtxRef,
        prepare_dlg: impl FnOnce(&mut KbgpPrepareHandle),
    ) {
        self.nodes.retain(|_, data| data.still_there);
        for node_data in self.nodes.values_mut() {
            node_data.still_there = false;
        }
        self.move_focus = None;
        self.activate = None;

        let mut handle = KbgpPrepareHandle {
            secs_after_first_movement: 0.6,
            secs_between_movements: 0.04,
            input: 0,
        };

        prepare_dlg(&mut handle);
        if handle.input != 0 {
            let mut effective_input = handle.input;
            let current_time = egui_ctx.input().time;
            if self.prev_input != handle.input {
                effective_input &= !self.prev_input;
                self.next_navigation = current_time + handle.secs_after_first_movement;
            } else if current_time < self.next_navigation {
                effective_input = 0;
            } else {
                self.next_navigation = current_time + handle.secs_between_movements;
            }

            if effective_input & INPUT_MASK_ACTIVATE != 0 {
                self.activate = egui_ctx.memory().focus();
            }

            match effective_input & INPUT_MASK_VERTICAL {
                INPUT_MASK_UP => {
                    self.move_focus(egui_ctx, |egui::Pos2 { x, y }| egui::Pos2 { x: -x, y: -y });
                }
                INPUT_MASK_DOWN => {
                    self.move_focus(egui_ctx, |p| p);
                }
                _ => {}
            }
            // Note: Doing transpose instead of rotation so that starting navigation without
            // anything focused will make left similar to up and right similar to down.
            match effective_input & INPUT_MASK_HORIZONTAL {
                INPUT_MASK_LEFT => {
                    self.move_focus(egui_ctx, |egui::Pos2 { x, y }| egui::Pos2 { x: -y, y: -x });
                }
                INPUT_MASK_RIGHT => {
                    self.move_focus(egui_ctx, |egui::Pos2 { x, y }| egui::Pos2 { x: y, y: x });
                }
                _ => {}
            }
        }
        self.prev_input = handle.input;
    }

    pub fn move_focus(
        &mut self,
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
        let transformed_nodes = self
            .nodes
            .iter()
            .map(|(id, data)| (id, transform_rect_downward(data.rect)));
        let focused_node_id = egui_ctx.memory().focus();
        let move_to = if let Some(focused_node_id) = focused_node_id {
            let focused_node_rect = if let Some(data) = self.nodes.get(&focused_node_id) {
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

pub trait KbgpEguiResponseExt {
    fn kbgp_initial_focus(self, kbgp: &mut Kbgp) -> Self;
    fn kbgp_navigation(self, kbgp: &mut Kbgp) -> Self;
    fn kbgp_activated(self, kbgp: &Kbgp) -> bool;
}

impl KbgpEguiResponseExt for egui::Response {
    fn kbgp_initial_focus(self, kbgp: &mut Kbgp) -> Self {
        if let Some(data) = kbgp.nodes.get(&self.id) {
            assert!(
                !data.still_there,
                "kbgp_navigation called before kbgp_initial_focus"
            );
        } else {
            self.request_focus();
        }
        self
    }

    fn kbgp_navigation(self, kbgp: &mut Kbgp) -> Self {
        if Some(self.id) == kbgp.move_focus || self.clicked() {
            self.request_focus();
        }
        kbgp.nodes.insert(
            self.id,
            NodeData {
                rect: self.rect,
                still_there: true,
            },
        );
        self
    }

    fn kbgp_activated(self, kbgp: &Kbgp) -> bool {
        self.clicked() || Some(self.id) == kbgp.activate
    }
}
