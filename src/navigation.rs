use std::any::Any;

use crate::egui;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::KbgpCommon;

const INPUT_MASK_UP: u8 = 1;
const INPUT_MASK_DOWN: u8 = 2;
const INPUT_MASK_VERTICAL: u8 = INPUT_MASK_UP | INPUT_MASK_DOWN;
const INPUT_MASK_LEFT: u8 = 4;
const INPUT_MASK_RIGHT: u8 = 8;
const INPUT_MASK_HORIZONTAL: u8 = INPUT_MASK_LEFT | INPUT_MASK_RIGHT;

const INPUT_MASK_CLICK: u8 = 16;
const INPUT_MASK_USER_ACTION: u8 = 32;

#[derive(Default)]
pub(crate) enum PendingReleaseState {
    #[default]
    Idle,
    NodeHeld {
        id: egui::Id,
        is_user_action: bool,
        user_action: Option<Box<dyn Any + Send + Sync>>,
    },
    NodeHoldReleased {
        id: egui::Id,
        user_action: Option<Box<dyn Any + Send + Sync>>,
    },
    GloballyHeld {
        user_action: Option<Box<dyn Any + Send + Sync>>,
    },
    GlobalHoldReleased {
        user_action: Box<dyn Any + Send + Sync>,
    },
    Invalidated {
        // `kbgp_clear_input` sets this to `true`, so that the invalidation could survive the
        // clearing.
        cooldown_frame: bool,
    },
}

#[derive(Default)]
pub(crate) struct KbgpNavigationState {
    pub(crate) prev_input: u8,
    pub(crate) pending_release_state: PendingReleaseState,
    pub(crate) next_navigation: f64,
    pub(crate) user_action: Option<Box<dyn Any + Send + Sync>>,
    pub(crate) focus_label: Option<Box<dyn Any + Send + Sync>>,
    pub(crate) next_frame_focus_label: Option<Box<dyn Any + Send + Sync>>,
    pub(crate) focus_on: Option<egui::Id>,
    pub(crate) last_focus: Option<egui::Id>,
    pub(crate) mouse_was_last_on: Option<egui::Id>,
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
    input: u8,
    user_action: Option<Box<dyn Any + Send + Sync>>,
}

impl KbgpPrepareNavigation {
    pub fn apply_action(&mut self, command: &KbgpNavCommand) {
        match command {
            KbgpNavCommand::NavigateUp => {
                self.input |= INPUT_MASK_UP;
            }
            KbgpNavCommand::NavigateDown => {
                self.input |= INPUT_MASK_DOWN;
            }
            KbgpNavCommand::NavigateLeft => {
                self.input |= INPUT_MASK_LEFT;
            }
            KbgpNavCommand::NavigateRight => {
                self.input |= INPUT_MASK_RIGHT;
            }
            KbgpNavCommand::Click => {
                self.input |= INPUT_MASK_CLICK;
            }
            KbgpNavCommand::User(action) => {
                self.user_action = Some(action());
                self.input |= INPUT_MASK_USER_ACTION;
            }
        }
    }

    /// Navigate the UI with the keyboard.
    ///
    /// Note: Unless
    /// [`kbgp_intercept_default_activation`](crate::kbgp_intercept_default_activation) is called,
    /// `mimic_default_activation` should be set to `true`.
    pub fn navigate_keyboard_by_binding(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        binding: &HashMap<KeyCode, KbgpNavCommand>,
        mimic_default_activation: bool,
    ) {
        for key in keys.get_pressed() {
            if let Some(action) = binding.get(key) {
                self.apply_action(action);
            }
        }
        if mimic_default_activation
            && [KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]
                .into_iter()
                .any(|key| keys.pressed(key))
        {
            self.input |= INPUT_MASK_CLICK;
        }
    }

    /// Navigate the UI with a gamepad.
    ///
    /// * Use both left stick and d-pad for navigation.
    pub fn navigate_gamepad_by_binding(
        &mut self,
        gamepad: &Gamepad,
        binding: &HashMap<GamepadButton, KbgpNavCommand>,
    ) {
        for (axis_type, action_for_negative, action_for_positive) in [
            (
                GamepadAxis::LeftStickX,
                KbgpNavCommand::NavigateLeft,
                KbgpNavCommand::NavigateRight,
            ),
            (
                GamepadAxis::LeftStickY,
                KbgpNavCommand::NavigateDown,
                KbgpNavCommand::NavigateUp,
            ),
        ] {
            if let Some(axis_value) = gamepad.get(axis_type) {
                if axis_value < -0.5 {
                    self.apply_action(&action_for_negative)
                } else if 0.5 < axis_value {
                    self.apply_action(&action_for_positive)
                }
            }
        }
        for button_type in gamepad.get_pressed() {
            if let Some(action) = binding.get(button_type) {
                self.apply_action(action);
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
        let mut handle = KbgpPrepareNavigation {
            secs_after_first_input: 0.6,
            secs_between_inputs: 0.04,
            input: 0,
            user_action: None,
        };

        prepare_dlg(&mut handle);
        let prev_user_action = self.user_action.take();
        if handle.input != 0 {
            let mut effective_input = handle.input;
            let current_time = egui_ctx.input(|input| input.time);
            if self.prev_input != handle.input {
                effective_input &= !self.prev_input;
                self.next_navigation = current_time + handle.secs_after_first_input;
            } else if current_time < self.next_navigation {
                effective_input = 0;
            } else {
                self.next_navigation = current_time + handle.secs_between_inputs;
            }

            if effective_input & INPUT_MASK_CLICK != 0 {
                egui_ctx.input_mut(|input| {
                    input.events.push(egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        modifiers: Default::default(),
                        repeat: false,
                    });
                });
            }

            if effective_input & INPUT_MASK_USER_ACTION != 0 {
                self.user_action = handle.user_action;
            }

            let mut move_focus_to = None;

            match effective_input & INPUT_MASK_VERTICAL {
                INPUT_MASK_UP => {
                    move_focus_to =
                        self.move_focus(common, egui_ctx, None, |egui::Pos2 { x, y }| egui::Pos2 {
                            x: -x,
                            y: -y,
                        });
                }
                INPUT_MASK_DOWN => {
                    move_focus_to = self.move_focus(common, egui_ctx, None, |p| p);
                }
                _ => {}
            }

            // Note: Doing transpose instead of rotation so that starting navigation without
            // anything focused will make left similar to up and right similar to down.
            match effective_input & INPUT_MASK_HORIZONTAL {
                INPUT_MASK_LEFT => {
                    move_focus_to =
                        self.move_focus(common, egui_ctx, move_focus_to, |egui::Pos2 { x, y }| {
                            egui::Pos2 { x: -y, y: -x }
                        });
                }
                INPUT_MASK_RIGHT => {
                    move_focus_to =
                        self.move_focus(common, egui_ctx, move_focus_to, |egui::Pos2 { x, y }| {
                            egui::Pos2 { x: y, y: x }
                        });
                }
                _ => {}
            }

            if let Some(move_focus) = move_focus_to {
                egui_ctx.memory_mut(|memory| memory.request_focus(move_focus));
            }
        }

        self.prev_input = handle.input;

        match &mut self.pending_release_state {
            PendingReleaseState::Idle => {
                self.pending_release_state = match handle.input
                    & (INPUT_MASK_CLICK | INPUT_MASK_USER_ACTION)
                {
                    0 => PendingReleaseState::Idle,
                    INPUT_MASK_CLICK => {
                        if let Some(current_focus) = egui_ctx.memory(|memory| memory.focused()) {
                            PendingReleaseState::NodeHeld {
                                id: current_focus,
                                is_user_action: false,
                                user_action: prev_user_action,
                            }
                        } else {
                            PendingReleaseState::Invalidated {
                                cooldown_frame: false,
                            }
                        }
                    }
                    INPUT_MASK_USER_ACTION => {
                        if let Some(current_focus) = egui_ctx.memory(|memory| memory.focused()) {
                            PendingReleaseState::NodeHeld {
                                id: current_focus,
                                is_user_action: true,
                                user_action: prev_user_action,
                            }
                        } else {
                            PendingReleaseState::GloballyHeld {
                                user_action: prev_user_action,
                            }
                        }
                    }
                    _ => PendingReleaseState::Invalidated {
                        cooldown_frame: false,
                    },
                }
            }
            PendingReleaseState::NodeHeld {
                id,
                is_user_action,
                user_action,
            } => {
                let current_focus = egui_ctx.memory(|memory| memory.focused());
                match handle.input & (INPUT_MASK_CLICK | INPUT_MASK_USER_ACTION) {
                    0 => {
                        if current_focus == Some(*id) {
                            self.pending_release_state = PendingReleaseState::NodeHoldReleased {
                                id: *id,
                                user_action: user_action.take().or(prev_user_action),
                            };
                        } else {
                            self.pending_release_state = PendingReleaseState::Idle;
                        }
                    }
                    INPUT_MASK_CLICK => {
                        if *is_user_action {
                            self.pending_release_state = PendingReleaseState::Invalidated {
                                cooldown_frame: false,
                            };
                        }
                    }
                    INPUT_MASK_USER_ACTION => {
                        if !*is_user_action {
                            self.pending_release_state = PendingReleaseState::Invalidated {
                                cooldown_frame: false,
                            };
                        } else if prev_user_action.is_some() {
                            *user_action = prev_user_action;
                        }
                    }
                    _ => {
                        self.pending_release_state = PendingReleaseState::Invalidated {
                            cooldown_frame: false,
                        };
                    }
                }
            }
            PendingReleaseState::NodeHoldReleased {
                id: _,
                user_action: _,
            } => {
                self.pending_release_state = PendingReleaseState::Idle;
            }
            PendingReleaseState::GloballyHeld { user_action } => {
                match handle.input & (INPUT_MASK_CLICK | INPUT_MASK_USER_ACTION) {
                    0 => {
                        if let Some(user_action) = user_action.take().or(prev_user_action) {
                            self.pending_release_state =
                                PendingReleaseState::GlobalHoldReleased { user_action };
                        } else {
                            self.pending_release_state = PendingReleaseState::Idle;
                        }
                    }
                    INPUT_MASK_USER_ACTION => {
                        if prev_user_action.is_some() {
                            *user_action = prev_user_action;
                        }
                    }
                    _ => {
                        self.pending_release_state = PendingReleaseState::Invalidated {
                            cooldown_frame: false,
                        };
                    }
                }
            }
            PendingReleaseState::GlobalHoldReleased { user_action: _ } => {
                self.pending_release_state = PendingReleaseState::Idle;
            }
            PendingReleaseState::Invalidated { cooldown_frame } => {
                if *cooldown_frame {
                    *cooldown_frame = false;
                } else if handle.input & INPUT_MASK_CLICK == 0 {
                    self.pending_release_state = PendingReleaseState::Idle;
                }
            }
        }
    }

    fn move_focus(
        &mut self,
        common: &KbgpCommon,
        egui_ctx: &egui::Context,
        move_from: Option<egui::Id>,
        transform_pos_downward: impl Fn(egui::Pos2) -> egui::Pos2,
    ) -> Option<egui::Id> {
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
        let focused_node_id = move_from.or_else(|| egui_ctx.memory(|memory| memory.focused()));
        if let Some(focused_node_id) = focused_node_id {
            let focused_node_rect = if let Some(data) = common.nodes.get(&focused_node_id) {
                transform_rect_downward(data.rect)
            } else {
                return Some(focused_node_id);
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
                .map(|(id, _)| *id)
        } else {
            transformed_nodes
                .map(|(id, rect)| (id, (rect.min.y, rect.min.x)))
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(id, _)| *id)
        }
    }
}

pub enum KbgpNavCommand {
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
    Click,
    /// Activeate a user defined command.
    ///
    /// This variant is tricky to construct directly - use [`KbgpNavCommand::user`] instead.
    ///
    /// User commands can be checked by using
    /// [`kbgp_user_action`](crate::KbgpEguiResponseExt::kbgp_user_action) or
    /// [`kbgp_activated`](crate::KbgpEguiResponseExt::kbgp_activated) on the widget and
    /// [`kbgp_user_action`](crate::KbgpEguiUiCtxExt::kbgp_user_action) on the UI handle.
    ///
    /// Each of these methods has a `_released` variation, that only "fires" after the key/button
    /// is released. This is useful for exiting menus, to avoid having the same key/button that was
    /// used to exit the menu registered as actual game input.
    User(Box<dyn 'static + Send + Sync + Fn() -> Box<dyn Any + Send + Sync>>),
}

impl KbgpNavCommand {
    /// Used to define user-commands.
    ///
    /// To define a user command, the result of this function should be bound to a key or a gamepad
    /// button in [`KbgpNavBindings`] which in turn should be placed inside the
    /// [`KbgpSettings`](crate::KbgpSettings) resource. Then a widget's
    /// [`kbgp_user_action`](crate::KbgpEguiResponseExt::kbgp_user_action) or
    /// [`kbgp_activated`](crate::KbgpEguiResponseExt::kbgp_activated) or the
    /// [`kbgp_user_action`](crate::KbgpEguiUiCtxExt::kbgp_user_action) on the UI handle can be
    /// used to determine if the player activated this action.
    ///
    /// Each of these methods has a `_released` variation, that only "fires" after the key/button
    /// is released. This is useful for exiting menus, to avoid having the same key/button that was
    /// used to exit the menu registered as actual game input.
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_egui::{EguiPrimaryContextPass, EguiContexts, EguiPlugin};
    /// use bevy_egui_kbgp::{egui, bevy_egui};
    /// use bevy_egui_kbgp::prelude::*;
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugins(EguiPlugin::default())
    ///         .add_plugins(KbgpPlugin)
    ///         .add_systems(EguiPrimaryContextPass, ui_system)
    ///         .insert_resource(KbgpSettings {
    ///             bindings: bevy_egui_kbgp::KbgpNavBindings::default()
    ///                 .with_key(KeyCode::Escape, KbgpNavCommand::user(UserAction::Exit))
    ///                 .with_key(KeyCode::KeyZ, KbgpNavCommand::user(UserAction::Special1))
    ///                 .with_key(KeyCode::KeyX, KbgpNavCommand::user(UserAction::Special2)),
    ///             ..Default::default()
    ///         })
    ///         .run();
    /// }
    ///
    /// #[derive(Clone)]
    /// enum UserAction {
    ///     Exit,
    ///     Special1,
    ///     Special2,
    /// }
    ///
    /// fn ui_system(
    ///     mut egui_context: EguiContexts,
    /// ) -> Result {
    ///     egui::CentralPanel::default().show(egui_context.ctx_mut()?, |ui| {
    ///         if matches!(ui.kbgp_user_action(), Some(UserAction::Exit)) {
    ///             println!("User wants to exit");
    ///         }
    ///         match ui.button("Button").kbgp_activated() {
    ///             KbgpNavActivation::Clicked => {
    ///                 println!("Regular button activation");
    ///             }
    ///             KbgpNavActivation::ClickedSecondary | KbgpNavActivation::User(UserAction::Special1) => {
    ///                 println!("Special button activation 1");
    ///             }
    ///             KbgpNavActivation::ClickedMiddle | KbgpNavActivation::User(UserAction::Special2) => {
    ///                 println!("Special button activation 2");
    ///             }
    ///             _ => {}
    ///         }
    ///     });
    ///     Ok(())
    /// }
    /// ```
    pub fn user<T: 'static + Clone + Send + Sync>(value: T) -> Self {
        Self::User(Box::new(move || Box::new(value.clone())))
    }
}

/// Input mapping for navigation.
pub struct KbgpNavBindings {
    /// The configured keyboard bindings.
    pub keyboard: HashMap<KeyCode, KbgpNavCommand>,
    /// The configured gamepad bindings.
    ///
    /// These are not limited to a specific gamepad, and are for buttons only - the axis behavior
    /// is hard coded. Note that in some environments the d-pad is treated as an axis.
    pub gamepad_buttons: HashMap<GamepadButton, KbgpNavCommand>,
}

impl Default for KbgpNavBindings {
    /// Create bindings with the default mappings.
    ///
    /// * Navigation: arrow keys, d-pad, left stick.
    /// * Activateion: Enter (egui builtin), Spacebar (also egui builtin), gamepad south button.
    fn default() -> Self {
        Self::empty()
            .with_arrow_keys_navigation()
            .with_gamepad_dpad_navigation_and_south_button_activation()
    }
}

impl KbgpNavBindings {
    /// Create empty bindings with no mapping.
    ///
    /// Gamepad axes will still be mapped, because their handling is hard coded. Note that in some
    /// environments the d-pad is treated as an axis.
    pub fn empty() -> Self {
        Self {
            keyboard: Default::default(),
            gamepad_buttons: Default::default(),
        }
    }

    /// Create bindings with the default mappings for gamepad.
    ///
    /// * Navigation: d-pad, left stick.
    /// * Activateion: gamepad south button.
    ///
    /// Use this instead of [`default`](Self::default) when
    /// [`disable_default_navigation`](crate::KbgpSettings::disable_default_navigation) is not set,
    /// otherwise both egui and KBGP will process the arrow keys, resulting in double-movement.
    pub fn default_gamepad_only() -> Self {
        Self::empty().with_gamepad_dpad_navigation_and_south_button_activation()
    }

    /// Bind the arrow keys for navigation.
    ///
    /// [`KbgpNavBindings::default`] already contains these mappings.
    pub fn bind_arrow_keys_navigation(&mut self) {
        self.bind_key(KeyCode::ArrowUp, KbgpNavCommand::NavigateUp);
        self.bind_key(KeyCode::ArrowDown, KbgpNavCommand::NavigateDown);
        self.bind_key(KeyCode::ArrowLeft, KbgpNavCommand::NavigateLeft);
        self.bind_key(KeyCode::ArrowRight, KbgpNavCommand::NavigateRight);
    }

    /// Bind the arrow keys for navigation.
    ///
    /// [`KbgpNavBindings::default`] already contains these mappings.
    pub fn with_arrow_keys_navigation(mut self) -> Self {
        self.bind_arrow_keys_navigation();
        self
    }

    /// Bind the gamepad's d-pad for navigation and south button for activation.
    ///
    /// [`KbgpNavBindings::default`] already contains these mappings.
    pub fn bind_gamepad_dpad_navigation_and_south_button_activation(&mut self) {
        self.bind_gamepad_button(GamepadButton::DPadUp, KbgpNavCommand::NavigateUp);
        self.bind_gamepad_button(GamepadButton::DPadDown, KbgpNavCommand::NavigateDown);
        self.bind_gamepad_button(GamepadButton::DPadLeft, KbgpNavCommand::NavigateLeft);
        self.bind_gamepad_button(GamepadButton::DPadRight, KbgpNavCommand::NavigateRight);
        self.bind_gamepad_button(GamepadButton::South, KbgpNavCommand::Click);
    }

    /// Bind the gamepad's d-pad for navigation and south button for activation.
    ///
    /// [`KbgpNavBindings::default`] already contains these mappings.
    pub fn with_gamepad_dpad_navigation_and_south_button_activation(mut self) -> Self {
        self.bind_gamepad_dpad_navigation_and_south_button_activation();
        self
    }

    /// Bind WASD for navigation.
    pub fn bind_wasd_navigation(&mut self) {
        self.bind_key(KeyCode::KeyW, KbgpNavCommand::NavigateUp);
        self.bind_key(KeyCode::KeyS, KbgpNavCommand::NavigateDown);
        self.bind_key(KeyCode::KeyA, KbgpNavCommand::NavigateLeft);
        self.bind_key(KeyCode::KeyD, KbgpNavCommand::NavigateRight);
    }

    /// Bind WASD for navigation.
    pub fn with_wasd_navigation(mut self) -> Self {
        self.bind_wasd_navigation();
        self
    }

    /// Bind a command to a keyboard key.
    pub fn bind_key(&mut self, key: KeyCode, command: KbgpNavCommand) {
        self.keyboard.insert(key, command);
    }

    /// Bind a command to a keyboard key.
    pub fn with_key(mut self, key: KeyCode, command: KbgpNavCommand) -> Self {
        self.bind_key(key, command);
        self
    }

    /// Bind a command to a gamepad button.
    pub fn bind_gamepad_button(&mut self, gamepad_button: GamepadButton, command: KbgpNavCommand) {
        self.gamepad_buttons.insert(gamepad_button, command);
    }

    /// Bind a command to a gamepad button.
    pub fn with_gamepad_button(
        mut self,
        gamepad_button: GamepadButton,
        command: KbgpNavCommand,
    ) -> Self {
        self.bind_gamepad_button(gamepad_button, command);
        self
    }
}

pub enum KbgpNavActivation<T> {
    /// The widget was not actiated this frame.
    None,
    /// The widget's primary function was activated.
    ///
    /// This means it was either left-clicked, or it was focused and the player pressed on Enter,
    /// Spacebar, the gamepad's south button (unless overriden), or some other key or button set in
    /// [`KbgpNavBindings`].
    Clicked,
    /// The widget was right-clicked.
    ClickedSecondary,
    /// The widget was middle-clicked.
    ClickedMiddle,
    /// A user action was activated when the focus was on this widget.
    User(T),
}
