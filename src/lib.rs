//! Improve the keyboard and gamepads usage for egui in Bevy.
//!
//! Usage:
//! * Add [`KbgpPlugin`].
//! * Use [the extension methods](crate::KbgpEguiResponseExt) on the egui widgets to add KBGP's
//!   functionality.
//! * Call [`ui.kbgp_clear_input`](crate::KbgpEguiUiCtxExt::kbgp_clear_input) when triggering a
//!   state transition as a response to a click on an egui widget. To control the focus in the new
//!   state, use [`kbgp_focus_label`](KbgpEguiResponseExt::kbgp_focus_label) (and
//!   [`kbgp_set_focus_label`](KbgpEguiUiCtxExt::kbgp_set_focus_label)) - otherwise egui will pick
//!   the widget to focus on (or elect to drop the focus entirely)
//! * To set special actions, see [the example here](crate::KbgpNavCommand::user). To avoid having
//!   to deal with both Bevy's input methods and KBGP's input, it's better to use these actions for
//!   entering the pause menu from within the game.
//! * Use [`kbgp_click_released`](KbgpEguiResponseExt::kbgp_click_released) instead of egui's
//!   `clicked` to register the button presss only when the user releases the key/button. This is
//!   useful for exiting menus, to avoid having the same key/button that was used to exit the menu
//!   registered as actual game input.
//!
//! ```no_run
//! use bevy_egui_kbgp::{egui, bevy_egui};
//! use bevy::prelude::*;
//! use bevy_egui::{EguiPrimaryContextPass, EguiContexts, EguiPlugin};
//! use bevy_egui_kbgp::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(EguiPlugin::default())
//!         .add_plugins(KbgpPlugin)
//!         .add_systems(EguiPrimaryContextPass, ui_system)
//!         .run();
//! }
//!
//! fn ui_system(
//!     mut egui_context: EguiContexts,
//!     keys: Res<ButtonInput<KeyCode>>,
//! ) -> Result {
//!     egui::CentralPanel::default().show(egui_context.ctx_mut()?, |ui| {
//!         if ui
//!             .button("Button")
//!             .kbgp_initial_focus()
//!             .kbgp_navigation()
//!             .clicked()
//!         {
//!             // Button action
//!         }
//!
//!         if let Some(input_selected_by_player) = ui
//!             .button("Set Input")
//!             .kbgp_navigation()
//!             .kbgp_pending_input()
//!         {
//!             // Do something with the input
//!         }
//!     });
//!     Ok(())
//! }
//! ```
//!
//! ## Creating Key-Setting UI
//!
//! Use functions like [`kbgp_pending_input`](crate::KbgpEguiResponseExt::kbgp_pending_input) to
//! convert a regular button to a key-setting button. When the players presses that button, they'll
//! be prompted to enter input from the keyboard, the mouse, or a gamepad. That input will be
//! returned as a [`KbgpInput`].
//!
//! [`kbgp_pending_chord`](crate::KbgpEguiResponseExt::kbgp_pending_chord) is similar, but prompts
//! the player to enter multiple keys instead of just one.
//!
//! Both functions have several variants that allow limiting the chords/keys accepted by that
//! button.
//!
//! By default, mouse wheel input is disabled. The reason is that mouse wheel events are a pain to
//! deal with, and most third party crates that ease input handling don't support them - so it's
//! better not to let the player select input that the game is unable to deal with.

pub use bevy_egui;
pub use bevy_egui::egui;

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use self::navigation::KbgpPrepareNavigation;
pub use self::navigation::{KbgpNavActivation, KbgpNavBindings, KbgpNavCommand};
use self::navigation::{KbgpNavigationState, PendingReleaseState};
use self::pending_input::KbgpPendingInputState;
pub use self::pending_input::{KbgpInputManualHandle, KbgpPreparePendingInput};

mod navigation;
mod pending_input;

pub mod prelude {
    pub use crate::kbgp_prepare;
    pub use crate::KbgpEguiResponseExt;
    pub use crate::KbgpEguiUiCtxExt;
    pub use crate::KbgpInput;
    pub use crate::KbgpInputSource;
    pub use crate::KbgpNavActivation;
    pub use crate::KbgpNavBindings;
    pub use crate::KbgpNavCommand;
    pub use crate::KbgpPlugin;
    pub use crate::KbgpSettings;
}

/// Adds KBGP input handling system and [`KbgpSettings`].
pub struct KbgpPlugin;

impl Plugin for KbgpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KbgpSettings::default());
        app.add_systems(
            EguiPrimaryContextPass,
            kbgp_system_default_input.after(bevy_egui::EguiPreUpdateSet::BeginPass),
        );
    }
}

/// General configuration resource for KBGP.
///
/// Note: [`KbgpPlugin`] will add the default settings, so custom settings should either be added
/// after the plugin or modified with a system. The default is to enable everything except the
/// mouse wheel.
#[derive(Resource)]
pub struct KbgpSettings {
    /// Whether or not egui's tab navigation should work
    pub disable_default_navigation: bool,
    /// Whether or not egui's Enter and Space should work. These keys can still be assigned with
    /// [`bindings`](KbgpSettings::bindings):
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_egui_kbgp::prelude::*;
    /// App::new()
    ///     // ...
    ///     .insert_resource(KbgpSettings {
    ///         disable_default_activation: true,
    ///         bindings: KbgpNavBindings::default()
    ///             .with_key(KeyCode::Space, KbgpNavCommand::Click)
    ///             .with_key(KeyCode::NumpadEnter, KbgpNavCommand::Click)
    ///             .with_key(KeyCode::Enter, KbgpNavCommand::Click),
    ///         ..Default::default()
    ///     })
    ///     // ...
    /// # ;
    /// ```
    pub disable_default_activation: bool,
    /// Whether or not to force that there is always an egui widget that has the focus.
    pub prevent_loss_of_focus: bool,
    /// Whether or not to transfer focus when the mouse moves into a widget.
    ///
    /// Only works for widgets marked with [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation).
    pub focus_on_mouse_movement: bool,
    /// Whether or not keyboard input is accepted for navigation and for chords.
    pub allow_keyboard: bool,
    /// Whether or not mouse buttons are accepted for chords.
    pub allow_mouse_buttons: bool,
    /// Whether or not mouse wheel is accepted for chords. Defaults to `false`.
    pub allow_mouse_wheel: bool,
    /// Whether or not mouse wheel sideways scrolling is accepted for chords. Defaults to `false`.
    pub allow_mouse_wheel_sideways: bool,
    /// Whether or not gamepads input is accepted for navigation and for chords.
    pub allow_gamepads: bool,
    /// Input mapping for navigation.
    pub bindings: KbgpNavBindings,
}

impl Default for KbgpSettings {
    fn default() -> Self {
        Self {
            disable_default_navigation: false,
            disable_default_activation: false,
            prevent_loss_of_focus: false,
            focus_on_mouse_movement: false,
            allow_keyboard: true,
            allow_mouse_buttons: true,
            allow_mouse_wheel: false,
            allow_mouse_wheel_sideways: false,
            allow_gamepads: true,
            bindings: Default::default(),
        }
    }
}

/// Object used to configure KBGP's behavior in [`kbgp_prepare`].
pub enum KbgpPrepare<'a> {
    Navigation(&'a mut KbgpPrepareNavigation),
    PendingInput(&'a mut KbgpPreparePendingInput),
}

#[derive(Default)]
struct Kbgp {
    common: KbgpCommon,
    state: KbgpState,
}

fn kbgp_get(egui_ctx: &egui::Context) -> std::sync::Arc<egui::mutex::Mutex<Kbgp>> {
    egui_ctx.memory_mut(|memory| {
        memory
            .data
            .get_temp_mut_or_default::<std::sync::Arc<egui::mutex::Mutex<Kbgp>>>(egui::Id::NULL)
            .clone()
    })
}

/// Must be called every frame, either manually or by using [`KbgpPlugin`].
///
/// Should be called between bevy_egui's input handling system and the system that generates the
/// UI - so in the `CoreStage::PreUpdate` stage after the `EguiSystem::ProcessInput` label.
///
/// The `prepare_dlg` argument is a closure that accepts a [`KbgpPrepare`], and used to:
///
/// * Register the input from the keyboard and the gamepads.
/// * Set preferences.
///
/// Typical usage:
///
/// ```no_run
/// # use bevy_egui_kbgp::bevy_egui;
/// # use bevy::prelude::*;
/// # use bevy_egui::{EguiContexts, EguiPlugin};
/// # use bevy_egui_kbgp::prelude::*;
/// # use bevy_egui_kbgp::KbgpPrepare;
/// fn custom_kbgp_system(
///     mut egui_context: EguiContexts,
///     keys: Res<ButtonInput<KeyCode>>,
///     gamepads: Query<(Entity, &Gamepad)>,
///     mouse_buttons: Res<ButtonInput<MouseButton>>,
///     settings: Res<KbgpSettings>,
/// ) -> Result {
///     kbgp_prepare(egui_context.ctx_mut()?, |prp| {
///         match prp {
///             KbgpPrepare::Navigation(prp) => {
///                 prp.navigate_keyboard_by_binding(&keys, &settings.bindings.keyboard, true);
///                 for (_, gamepad) in gamepads.iter() {
///                     prp.navigate_gamepad_by_binding(gamepad, &settings.bindings.gamepad_buttons);
///                 }
///             }
///             KbgpPrepare::PendingInput(prp) => {
///                 prp.accept_keyboard_input(&keys);
///                 prp.accept_mouse_buttons_input(&mouse_buttons);
///                 for (gamepad_entity, gamepad) in gamepads.iter() {
///                     prp.accept_gamepad_input(gamepad_entity, gamepad);
///                 }
///             }
///         }
///     });
///     Ok(())
/// }
/// ```
pub fn kbgp_prepare(egui_ctx: &egui::Context, prepare_dlg: impl FnOnce(KbgpPrepare<'_>)) {
    let kbgp = kbgp_get(egui_ctx);
    let mut kbgp = kbgp.lock();
    // Since Bevy is allow to reorder systems mid-run, there is a risk that the KBGP prepare system
    // run twice between egui drawing systems. The stale counter allows up to two such invocations
    // - after that it assumes the widget is no longer drawn.
    kbgp.common.nodes.retain(|_, data| data.seen_this_frame);
    for node_data in kbgp.common.nodes.values_mut() {
        node_data.seen_this_frame = false;
    }
    let Kbgp { common, state } = &mut *kbgp;
    match state {
        KbgpState::Navigation(state) => {
            state.prepare(common, egui_ctx, |prp| {
                prepare_dlg(KbgpPrepare::Navigation(prp))
            });
            if let Some(focus_on) = state.focus_on.take() {
                egui_ctx.memory_mut(|memory| memory.request_focus(focus_on));
            }
            state.focus_label = state.next_frame_focus_label.take();
            if common.nodes.is_empty() && state.focus_label.is_none() {
                state.focus_label = Some(Box::new(KbgpInitialFocusLabel));
            }
        }
        KbgpState::PendingInput(state) => {
            state.prepare(common, egui_ctx, |prp| {
                prepare_dlg(KbgpPrepare::PendingInput(prp))
            });
            if common.nodes.is_empty() {
                kbgp.state = KbgpState::Navigation(Default::default());
            }
        }
    }
}

/// Cancel's any tab-based navigation egui did in its `BeginFrame`.
pub fn kbgp_intercept_default_navigation(egui_ctx: &egui::Context) {
    egui_ctx.memory_mut(|memory| {
        if let Some(focus) = memory.focused() {
            memory.set_focus_lock_filter(
                focus,
                egui::EventFilter {
                    tab: true,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: true,
                },
            );
        }
    });
}

/// Hide from egui Space and Enter key events.
///
/// KBGP gets its keys directly from Bevy, so this function will not hide these keys from it.
pub fn kbgp_intercept_default_activation(egui_ctx: &egui::Context) {
    egui_ctx.input_mut(|input| {
        input.events.retain(|evt| match evt {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                modifiers: _,
                repeat: _,
            } => !matches!(key, egui::Key::Enter | egui::Key::Space),
            _ => true,
        });
    });
}

/// Make sure there is always an egui widget that has the focus.
pub fn kbgp_prevent_loss_of_focus(egui_ctx: &egui::Context) {
    let kbgp = kbgp_get(egui_ctx);
    let mut kbgp = kbgp.lock();

    match &mut kbgp.state {
        KbgpState::PendingInput(_) => {}
        KbgpState::Navigation(state) => {
            let current_focus = egui_ctx.memory(|memory| memory.focused());
            if let Some(current_focus) = current_focus {
                state.last_focus = Some(current_focus);
            } else if let Some(last_focus) = state.last_focus.take() {
                egui_ctx.memory_mut(|memory| memory.request_focus(last_focus));
            }
        }
    }
}

/// Transfer focus when the mouse moves into a widget.
///
/// Only works for widgets marked with [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation).
pub fn kbgp_focus_on_mouse_movement(egui_ctx: &egui::Context) {
    let kbgp = kbgp_get(egui_ctx);
    let mut kbgp = kbgp.lock();

    let Kbgp { common, state } = &mut *kbgp;

    match state {
        KbgpState::PendingInput(_) => {}
        KbgpState::Navigation(state) => {
            let node_at_pos = egui_ctx.input(|input| {
                input.pointer.interact_pos().and_then(|pos| {
                    common.nodes.iter().find_map(|(node_id, node_data)| {
                        node_data.rect.contains(pos).then_some(*node_id)
                    })
                })
            });
            if node_at_pos != state.mouse_was_last_on {
                state.mouse_was_last_on = node_at_pos;
                if let Some(node_at_pos) = node_at_pos {
                    egui_ctx.memory_mut(|memory| memory.request_focus(node_at_pos));
                }
            }
        }
    }
}

/// System that operates KBGP with the default input scheme.
///
/// * Keyboard:
///   * Arrow keys - navigation.
///   * egui already uses Space and Enter for widget activation.
/// * Gamepad:
///   * DPad - navigation.
///   * Left stick - navigation.
///   * South face button (depends on model - usually X or A): widget activation.
#[allow(clippy::too_many_arguments)]
fn kbgp_system_default_input(
    mut egui_context: EguiContexts,
    settings: Res<KbgpSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    gamepads: Query<(Entity, &Gamepad)>,
) -> Result {
    let egui_ctx = egui_context.ctx_mut()?;
    if settings.disable_default_navigation {
        kbgp_intercept_default_navigation(egui_ctx);
    }
    if settings.disable_default_activation {
        kbgp_intercept_default_activation(egui_ctx);
    }
    if settings.prevent_loss_of_focus {
        kbgp_prevent_loss_of_focus(egui_ctx);
    }
    if settings.focus_on_mouse_movement {
        kbgp_focus_on_mouse_movement(egui_ctx);
    }

    kbgp_prepare(egui_ctx, |prp| match prp {
        KbgpPrepare::Navigation(prp) => {
            if settings.allow_keyboard {
                prp.navigate_keyboard_by_binding(
                    &keys,
                    &settings.bindings.keyboard,
                    !settings.disable_default_activation,
                );
            }
            if settings.allow_gamepads {
                for (_, gamepad) in gamepads.iter() {
                    prp.navigate_gamepad_by_binding(gamepad, &settings.bindings.gamepad_buttons);
                }
            }
        }
        KbgpPrepare::PendingInput(prp) => {
            if settings.allow_keyboard {
                prp.accept_keyboard_input(&keys);
            }
            if settings.allow_mouse_buttons {
                prp.accept_mouse_buttons_input(&mouse_buttons);
            }
            if settings.allow_mouse_wheel || settings.allow_mouse_wheel_sideways {
                for event in mouse_wheel_events.read() {
                    prp.accept_mouse_wheel_event(
                        event,
                        settings.allow_mouse_wheel,
                        settings.allow_mouse_wheel_sideways,
                    );
                }
            }
            if settings.allow_gamepads {
                for (gamepad_entity, gamepad) in gamepads.iter() {
                    prp.accept_gamepad_input(gamepad_entity, gamepad);
                }
            }
        }
    });
    Ok(())
}

#[derive(Default)]
struct KbgpCommon {
    nodes: HashMap<egui::Id, NodeData>,
}

enum KbgpState {
    Navigation(KbgpNavigationState),
    PendingInput(KbgpPendingInputState),
}

impl Default for KbgpState {
    fn default() -> Self {
        Self::Navigation(Default::default())
    }
}

#[derive(Debug)]
struct NodeData {
    rect: egui::Rect,
    seen_this_frame: bool,
}

#[derive(PartialEq)]
struct KbgpInitialFocusLabel;

/// Extensions for egui's `Response` to activate KBGP's functionality.
///
/// ```no_run
/// # use bevy_egui_kbgp::egui;
/// # use bevy::prelude::*;
/// # use bevy_egui_kbgp::prelude::*;
/// # let ui: egui::Ui = todo!();
/// if ui
///     .button("My Button")
///     .kbgp_initial_focus() // focus on this button when starting the UI
///     .kbgp_navigation() // navigate to and from this button with keyboard/gamepad
///     .clicked()
/// {
///     // ...
/// }
/// ```
pub trait KbgpEguiResponseExt: Sized {
    /// Focus on this widget when [`kbgp_set_focus_label`](KbgpEguiUiCtxExt::kbgp_set_focus_label)
    /// is called with the same label.
    ///
    /// This will only happen if `kbgp_set_focus_label` was called in the previous frame. A single
    /// widget can be marked with multiple labels by calling `kbgp_focus_label` multiple times.
    ///
    /// ```no_run
    /// # use bevy_egui_kbgp::egui;
    /// # use bevy_egui_kbgp::prelude::*;
    /// # let ui: egui::Ui = todo!();
    /// #[derive(PartialEq)]
    /// enum FocusLabel {
    ///     Left,
    ///     Right,
    /// }
    /// if ui
    ///     .button("Focus >")
    ///     .kbgp_navigation()
    ///     .kbgp_focus_label(FocusLabel::Left)
    ///     .clicked()
    /// {
    ///     ui.kbgp_set_focus_label(FocusLabel::Right);
    /// }
    /// if ui
    ///     .button("< Focus")
    ///     .kbgp_navigation()
    ///     .kbgp_focus_label(FocusLabel::Right)
    ///     .clicked()
    /// {
    ///     ui.kbgp_set_focus_label(FocusLabel::Left);
    /// }
    /// ```
    fn kbgp_focus_label<T: 'static + PartialEq<T>>(self, label: T) -> Self;

    /// When the UI is first created, focus on this widget.
    ///
    /// Note that if [`kbgp_set_focus_label`](KbgpEguiUiCtxExt::kbgp_set_focus_label) was called in
    /// the previous frame the widget marked with
    /// [`kbgp_focus_label`](KbgpEguiResponseExt::kbgp_focus_label) will receive focus instead. A
    /// single widget can be marked with both `kbgp_focus_label` and `kbgp_initial_focus`.
    fn kbgp_initial_focus(self) -> Self {
        self.kbgp_focus_label(KbgpInitialFocusLabel)
    }

    /// Navigate to and from this widget.
    fn kbgp_navigation(self) -> Self;

    /// Check if the player pressed a user action button while focused on this widget.
    ///
    /// ```no_run
    /// # use bevy_egui_kbgp::egui;
    /// # use bevy_egui_kbgp::prelude::*;
    /// # let ui: egui::Ui = todo!();
    /// # #[derive(Clone)]
    /// # enum MyUserAction { Action1, Action2 }
    /// match ui.button("Button").kbgp_user_action() {
    ///     None => {}
    ///     Some(MyUserAction::Action1) => println!("User action 1"),
    ///     Some(MyUserAction::Action2) => println!("User action 2"),
    /// }
    /// ```
    fn kbgp_user_action<T: 'static + Clone>(&self) -> Option<T>;

    /// Check if the player activated this widget or pressed a user action button while focused on
    /// it.
    ///
    /// ```no_run
    /// # use bevy_egui_kbgp::egui;
    /// # use bevy_egui_kbgp::prelude::*;
    /// # let ui: egui::Ui = todo!();
    /// # #[derive(Clone)]
    /// # enum SpecialAction { Special1, Special2 }
    /// match ui.button("Button").kbgp_activated() {
    ///     KbgpNavActivation::Clicked => println!("Regular activateion"),
    ///     KbgpNavActivation::ClickedSecondary | KbgpNavActivation::User(SpecialAction::Special1) => println!("Special activateion 1"),
    ///     KbgpNavActivation::ClickedMiddle | KbgpNavActivation::User(SpecialAction::Special2) => println!("Special activateion 2"),
    ///     _ => {}
    /// }
    /// ```
    fn kbgp_activated<T: 'static + Clone>(&self) -> KbgpNavActivation<T>;

    /// Similar to [`kbgp_activated`](Self::kbgp_activated), but only returns a
    /// non-[`KbgpNavActivation::None`] value when the key/button is released.
    fn kbgp_activate_released<T: 'static + Clone>(&self) -> KbgpNavActivation<T>;

    /// Similar to egui's `clicked`, but only returns `true` when the key/button is released.
    fn kbgp_click_released(&self) -> bool;

    /// Similar to [`kbgp_user_action`](Self::kbgp_user_action), but only returns `Some` when the
    /// key/button is released.
    fn kbgp_user_action_released<T: 'static + Clone>(&self) -> Option<T>;

    /// Accept a single key/button input from this widget.
    ///
    /// Must be called on widgets that had
    /// [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) called on them.
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
    ///         .insert_resource(JumpInput(KbgpInput::Keyboard(KeyCode::Space)))
    ///         .run();
    /// }
    ///
    /// #[derive(Resource)]
    /// struct JumpInput(KbgpInput);
    ///
    /// fn ui_system(
    ///     mut egui_context: EguiContexts,
    ///     mut jump_input: ResMut<JumpInput>,
    /// ) -> Result {
    ///     egui::CentralPanel::default().show(egui_context.ctx_mut()?, |ui| {
    ///         ui.horizontal(|ui| {
    ///             ui.label("Set button for jumping");
    ///             if let Some(new_jump_input) = ui.button(format!("{}", jump_input.0))
    ///                 .kbgp_navigation()
    ///                 .kbgp_pending_input()
    ///             {
    ///                 jump_input.0 = new_jump_input;
    ///             }
    ///         });
    ///     });
    ///     Ok(())
    /// }
    fn kbgp_pending_input(&self) -> Option<KbgpInput>;

    /// Accept a single key/button input from this widget, limited to a specific input source.
    fn kbgp_pending_input_of_source(&self, source: KbgpInputSource) -> Option<KbgpInput>;

    /// Accept a single key/button input from this widget, with the ability to filter which inputs
    /// to accept.
    fn kbgp_pending_input_vetted(&self, pred: impl FnMut(KbgpInput) -> bool) -> Option<KbgpInput>;

    /// Accept a chord of key/button inputs from this widget.
    ///
    /// Must be called on widgets that had
    /// [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) called on them.
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
    ///         .insert_resource(JumpChord(vec![KbgpInput::Keyboard(KeyCode::Space)]))
    ///         .run();
    /// }
    ///
    /// #[derive(Resource)]
    /// struct JumpChord(Vec<KbgpInput>);
    ///
    /// fn ui_system(
    ///     mut egui_context: EguiContexts,
    ///     mut jump_chord: ResMut<JumpChord>,
    /// ) -> Result {
    ///     egui::CentralPanel::default().show(egui_context.ctx_mut()?, |ui| {
    ///         ui.horizontal(|ui| {
    ///             ui.label("Set chord of buttons for jumping");
    ///             if let Some(new_jump_chord) = ui
    ///                 .button(KbgpInput::format_chord(jump_chord.0.iter().cloned()))
    ///                 .kbgp_navigation()
    ///                 .kbgp_pending_chord()
    ///             {
    ///                 jump_chord.0 = new_jump_chord.into_iter().collect();
    ///             }
    ///         });
    ///     });
    ///     Ok(())
    /// }
    fn kbgp_pending_chord(&self) -> Option<HashSet<KbgpInput>>;

    /// Accept a chord of key/button inputs from this widget, limited to a specific input source.
    fn kbgp_pending_chord_of_source(&self, source: KbgpInputSource) -> Option<HashSet<KbgpInput>>;

    /// Accept a chord of key/button inputs from this widget, where all inputs are from the same
    /// source.
    ///
    /// "Same source" means either all the inputs are from the same gamepad, or all the inputs are
    /// from the keyboard and the mouse.
    fn kbgp_pending_chord_same_source(&self) -> Option<HashSet<KbgpInput>>;

    /// Accept a chord of key/button inputs from this widget, with the ability to filter which
    /// inputs to accept.
    ///
    /// The predicate accepts as a first argument the inputs that already participate in the chord,
    /// to allow vetting the new input based on them.
    fn kbgp_pending_chord_vetted(
        &self,
        pred: impl FnMut(&HashSet<KbgpInput>, KbgpInput) -> bool,
    ) -> Option<HashSet<KbgpInput>>;

    /// Helper for manually implementing custom methods for input-setting
    ///
    /// Inside the delegate, one would usually:
    /// * Call
    ///   [`process_new_input`](crate::pending_input::KbgpInputManualHandle::process_new_input) to
    ///   decide which new input to register.
    /// * Call
    ///   [`show_current_chord`](crate::pending_input::KbgpInputManualHandle::show_current_chord)
    ///   to show the tooltip, or generate some other visual cue.
    /// * Return `None` if the player did not finish entering the input.
    fn kbgp_pending_input_manual<T>(
        &self,
        dlg: impl FnOnce(&Self, KbgpInputManualHandle) -> Option<T>,
    ) -> Option<T>;
}

impl KbgpEguiResponseExt for egui::Response {
    fn kbgp_focus_label<T: 'static + PartialEq<T>>(self, label: T) -> Self {
        let kbgp = kbgp_get(&self.ctx);
        let mut kbgp = kbgp.lock();
        match &mut kbgp.state {
            KbgpState::Navigation(state) => {
                if let Some(focus_label) = &state.focus_label {
                    if let Some(focus_label) = focus_label.downcast_ref::<T>() {
                        if focus_label == &label {
                            state.focus_label = None;
                            state.focus_on = Some(self.id);
                        }
                    }
                }
            }
            KbgpState::PendingInput(_) => {}
        }
        self
    }

    //fn kbgp_initial_focus(self) -> Self {
    //let kbgp = kbgp_get(&self.ctx);
    //let mut kbgp = kbgp.lock();
    //match &mut kbgp.state {
    //KbgpState::Inactive(state) => {
    //state.focus_on = Some(self.id);
    //}
    //KbgpState::Navigation(_) => {}
    //KbgpState::PendingInput(_) => {}
    //}
    //self
    //}

    fn kbgp_navigation(self) -> Self {
        let kbgp = kbgp_get(&self.ctx);
        let mut kbgp = kbgp.lock();
        kbgp.common.nodes.insert(
            self.id,
            NodeData {
                rect: self.rect,
                seen_this_frame: true,
            },
        );
        self
    }

    fn kbgp_user_action<T: 'static + Clone>(&self) -> Option<T> {
        if self.has_focus() {
            self.ctx.kbgp_user_action()
        } else {
            None
        }
    }

    fn kbgp_activated<T: 'static + Clone>(&self) -> KbgpNavActivation<T> {
        if self.clicked() {
            KbgpNavActivation::Clicked
        } else if self.secondary_clicked() {
            KbgpNavActivation::ClickedSecondary
        } else if self.middle_clicked() {
            KbgpNavActivation::ClickedMiddle
        } else if let Some(action) = self.kbgp_user_action() {
            KbgpNavActivation::User(action)
        } else {
            KbgpNavActivation::None
        }
    }

    fn kbgp_activate_released<T: 'static + Clone>(&self) -> KbgpNavActivation<T> {
        if self.kbgp_click_released() {
            KbgpNavActivation::Clicked
        } else if self.secondary_clicked() {
            KbgpNavActivation::ClickedSecondary
        } else if self.middle_clicked() {
            KbgpNavActivation::ClickedMiddle
        } else if let Some(action) = self.kbgp_user_action_released() {
            KbgpNavActivation::User(action)
        } else {
            KbgpNavActivation::None
        }
    }

    fn kbgp_click_released(&self) -> bool {
        let kbgp = kbgp_get(&self.ctx);
        let kbgp = kbgp.lock();
        if let KbgpState::Navigation(state) = &kbgp.state {
            if let navigation::PendingReleaseState::NodeHoldReleased {
                id,
                user_action: None,
            } = &state.pending_release_state
            {
                return *id == self.id;
            }
        }
        // Otherwise it would not accept mouse clicks
        if self.hovered() && self.ctx.input(|input| input.pointer.primary_released()) {
            return true;
        }
        false
    }

    fn kbgp_user_action_released<T: 'static + Clone>(&self) -> Option<T> {
        if self.has_focus() {
            let kbgp = kbgp_get(&self.ctx);
            let kbgp = kbgp.lock();
            if let KbgpState::Navigation(state) = &kbgp.state {
                if let navigation::PendingReleaseState::NodeHoldReleased {
                    id,
                    user_action: Some(user_action),
                } = &state.pending_release_state
                {
                    if *id == self.id {
                        return user_action.downcast_ref().cloned();
                    }
                }
            }
        }
        None
    }

    fn kbgp_pending_input_manual<T>(
        &self,
        dlg: impl FnOnce(&Self, KbgpInputManualHandle) -> Option<T>,
    ) -> Option<T> {
        let kbgp = kbgp_get(&self.ctx);
        let mut kbgp = kbgp.lock();
        match &mut kbgp.state {
            KbgpState::Navigation(_) => {
                if self.clicked() {
                    kbgp.state = KbgpState::PendingInput(KbgpPendingInputState::new(self.id));
                }
                None
            }
            KbgpState::PendingInput(state) => {
                if state.acceptor_id != self.id {
                    return None;
                }
                self.request_focus();
                self.ctx.memory_mut(|memory| {
                    memory.set_focus_lock_filter(
                        self.id,
                        egui::EventFilter {
                            tab: true,
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            escape: true,
                        },
                    )
                });
                let handle = KbgpInputManualHandle { state };
                let result = dlg(self, handle);
                if result.is_some() {
                    kbgp.state = KbgpState::Navigation(KbgpNavigationState::default());
                }
                result
            }
        }
    }

    fn kbgp_pending_input(&self) -> Option<KbgpInput> {
        self.kbgp_pending_input_vetted(|_| true)
    }

    fn kbgp_pending_input_of_source(&self, source: KbgpInputSource) -> Option<KbgpInput> {
        self.kbgp_pending_input_vetted(|input| input.get_source() == source)
    }

    fn kbgp_pending_input_vetted(
        &self,
        mut pred: impl FnMut(KbgpInput) -> bool,
    ) -> Option<KbgpInput> {
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|hnd, input| hnd.received_input().is_empty() && pred(input));
            hnd.show_current_chord(response);
            if hnd
                .input_this_frame()
                .any(|inp| hnd.received_input().contains(&inp))
            {
                None
            } else {
                let mut it = hnd.received_input().iter();
                let single_input = it.next();
                assert!(
                    it.next().is_none(),
                    "More than one input in chord, but limit is 1"
                );
                // This will not be empty and we'll return a value if and only if there was some
                // input in received_input.
                single_input.cloned()
            }
        })
    }

    fn kbgp_pending_chord(&self) -> Option<HashSet<KbgpInput>> {
        self.kbgp_pending_chord_vetted(|_, _| true)
    }

    fn kbgp_pending_chord_of_source(&self, source: KbgpInputSource) -> Option<HashSet<KbgpInput>> {
        self.kbgp_pending_chord_vetted(|_, input| input.get_source() == source)
    }

    fn kbgp_pending_chord_same_source(&self) -> Option<HashSet<KbgpInput>> {
        self.kbgp_pending_chord_vetted(|existing, input| {
            if let Some(existing_input) = existing.iter().next() {
                input.get_source() == existing_input.get_source()
            } else {
                true
            }
        })
    }

    fn kbgp_pending_chord_vetted(
        &self,
        mut pred: impl FnMut(&HashSet<KbgpInput>, KbgpInput) -> bool,
    ) -> Option<HashSet<KbgpInput>> {
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|hnd, input| pred(hnd.received_input(), input));
            hnd.show_current_chord(response);
            if hnd.input_this_frame().any(|_| true) || hnd.received_input().is_empty() {
                None
            } else {
                Some(hnd.received_input().clone())
            }
        })
    }
}

/// Input from the keyboard or from a gamepad.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum KbgpInput {
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    MouseWheelUp,
    MouseWheelDown,
    MouseWheelLeft,
    MouseWheelRight,
    GamepadAxisPositive(Entity, GamepadAxis),
    GamepadAxisNegative(Entity, GamepadAxis),
    GamepadButton(Entity, GamepadButton),
}

impl core::fmt::Display for KbgpInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KbgpInput::Keyboard(key) => write!(f, "{key:?}")?,
            KbgpInput::MouseButton(MouseButton::Other(button)) => {
                write!(f, "MouseButton{button:?}")?
            }
            KbgpInput::MouseButton(button) => write!(f, "Mouse{button:?}")?,
            KbgpInput::MouseWheelUp => write!(f, "MouseScrollUp")?,
            KbgpInput::MouseWheelDown => write!(f, "MouseScrollDown")?,
            KbgpInput::MouseWheelLeft => write!(f, "MouseScrollLeft")?,
            KbgpInput::MouseWheelRight => write!(f, "MouseScrollRight")?,
            KbgpInput::GamepadButton(entity, gamepad_button) => {
                write!(f, "[{entity}]{gamepad_button:?}")?
            }
            KbgpInput::GamepadAxisPositive(entity, gamepad_axis) => {
                write!(f, "[{entity}]{gamepad_axis:?}")?
            }
            KbgpInput::GamepadAxisNegative(entity, gamepad_axis) => {
                write!(f, "[{entity}]-{gamepad_axis:?}")?
            }
        }
        Ok(())
    }
}

impl KbgpInput {
    /// Create a string that describes a chord of multiple inputs.
    pub fn format_chord(chord: impl Iterator<Item = Self>) -> String {
        let mut chord_text = String::new();
        for input in chord {
            use std::fmt::Write;
            if !chord_text.is_empty() {
                write!(&mut chord_text, " & ").unwrap();
            }
            write!(&mut chord_text, "{input}").unwrap();
        }
        chord_text
    }

    /// Return the source responsible for this input.
    pub fn get_source(&self) -> KbgpInputSource {
        match self {
            KbgpInput::Keyboard(_) => KbgpInputSource::KeyboardAndMouse,
            KbgpInput::MouseButton(_) => KbgpInputSource::KeyboardAndMouse,
            KbgpInput::MouseWheelUp => KbgpInputSource::KeyboardAndMouse,
            KbgpInput::MouseWheelDown => KbgpInputSource::KeyboardAndMouse,
            KbgpInput::MouseWheelLeft => KbgpInputSource::KeyboardAndMouse,
            KbgpInput::MouseWheelRight => KbgpInputSource::KeyboardAndMouse,
            KbgpInput::GamepadAxisPositive(entity, _) => KbgpInputSource::Gamepad(*entity),
            KbgpInput::GamepadAxisNegative(entity, _) => KbgpInputSource::Gamepad(*entity),
            KbgpInput::GamepadButton(entity, _) => KbgpInputSource::Gamepad(*entity),
        }
    }
}

/// Input from the keyboard or from a gamepad.
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum KbgpInputSource {
    KeyboardAndMouse,
    Gamepad(Entity),
}

/// A source of input for chords
impl core::fmt::Display for KbgpInputSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KbgpInputSource::KeyboardAndMouse => write!(f, "Keyboard&Mouse"),
            KbgpInputSource::Gamepad(entity) => write!(f, "Gamepad {entity}"),
        }
    }
}

impl KbgpInputSource {
    /// The gamepad of the source, of `None` if the source is keyboard or mouse.
    pub fn gamepad(&self) -> Option<Entity> {
        match self {
            KbgpInputSource::KeyboardAndMouse => None,
            KbgpInputSource::Gamepad(entity) => Some(*entity),
        }
    }
}

/// Extensions for egui's `UI` and Context to activate KBGP's functionality.
pub trait KbgpEguiUiCtxExt {
    /// Needs to be called when triggering state transition from egui.
    ///
    /// Otherwise, the same player input that triggered the transition will be applied again to the
    /// GUI in the new state.
    fn kbgp_clear_input(&self);

    /// Focus on the widget that called [`kbgp_focus_label`](KbgpEguiResponseExt::kbgp_focus_label)
    /// with the same label.
    ///
    /// This will only happen on the next frame.
    fn kbgp_set_focus_label<T: 'static + Send + Sync>(&self, label: T);

    /// Check if the player pressed a user action button.
    ///
    /// Note that if the focus is on a widget that handles a user action, it will be reported both
    /// by that widget's [`kbgp_user_action`](crate::KbgpEguiResponseExt::kbgp_user_action) or
    /// [`kbgp_activated`](crate::KbgpEguiResponseExt::kbgp_activated) and by this method.
    ///
    /// ```no_run
    /// # use bevy_egui_kbgp::egui;
    /// # use bevy_egui_kbgp::prelude::*;
    /// # let ui: egui::Ui = todo!();
    /// # #[derive(Clone)]
    /// # enum MyUserAction { Action1, Action2 }
    /// match ui.kbgp_user_action() {
    ///     None => {}
    ///     Some(MyUserAction::Action1) => println!("User action 1"),
    ///     Some(MyUserAction::Action2) => println!("User action 2"),
    /// }
    /// ```
    fn kbgp_user_action<T: 'static + Clone>(&self) -> Option<T>;

    /// Similar to [`kbgp_user_action`](Self::kbgp_user_action), but only returns `Some` when the
    /// key/button is released.
    fn kbgp_user_action_released<T: 'static + Clone>(&self) -> Option<T>;
}

impl KbgpEguiUiCtxExt for egui::Ui {
    fn kbgp_clear_input(&self) {
        self.ctx().kbgp_clear_input()
    }

    fn kbgp_set_focus_label<T: 'static + Send + Sync>(&self, label: T) {
        self.ctx().kbgp_set_focus_label(label);
    }

    fn kbgp_user_action<T: 'static + Clone>(&self) -> Option<T> {
        self.ctx().kbgp_user_action()
    }

    fn kbgp_user_action_released<T: 'static + Clone>(&self) -> Option<T> {
        self.ctx().kbgp_user_action_released()
    }
}

impl KbgpEguiUiCtxExt for egui::Context {
    fn kbgp_clear_input(&self) {
        let kbgp = kbgp_get(self);
        let mut kbgp = kbgp.lock();
        match &mut kbgp.state {
            KbgpState::PendingInput(_) => {}
            KbgpState::Navigation(state) => {
                state.user_action = None;
                state.pending_release_state = PendingReleaseState::Invalidated {
                    cooldown_frame: true,
                };
            }
        }

        self.input_mut(|input| {
            input.pointer = Default::default();
            #[allow(clippy::match_like_matches_macro)]
            input.events.retain(|event| match event {
                egui::Event::Key {
                    key: egui::Key::Space | egui::Key::Enter,
                    physical_key: _,
                    pressed: true,
                    modifiers: _,
                    repeat: _,
                } => false,
                egui::Event::PointerButton {
                    pos: _,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: _,
                } => false,
                _ => true,
            });
        });
    }

    fn kbgp_set_focus_label<T: 'static + Send + Sync>(&self, label: T) {
        let kbgp = kbgp_get(self);
        let mut kbgp = kbgp.lock();
        match &mut kbgp.state {
            KbgpState::PendingInput(_) => {}
            KbgpState::Navigation(state) => {
                state.next_frame_focus_label = Some(Box::new(label));
            }
        }
    }

    fn kbgp_user_action<T: 'static + Clone>(&self) -> Option<T> {
        let kbgp = kbgp_get(self);
        let kbgp = kbgp.lock();
        match &kbgp.state {
            KbgpState::PendingInput(_) => None,
            KbgpState::Navigation(state) => state.user_action.as_ref()?.downcast_ref().cloned(),
        }
    }

    fn kbgp_user_action_released<T: 'static + Clone>(&self) -> Option<T> {
        let kbgp = kbgp_get(self);
        let kbgp = kbgp.lock();
        match &kbgp.state {
            KbgpState::PendingInput(_) => None,
            KbgpState::Navigation(state) => match &state.pending_release_state {
                navigation::PendingReleaseState::Idle => None,
                navigation::PendingReleaseState::NodeHeld { .. } => None,
                navigation::PendingReleaseState::NodeHoldReleased { id: _, user_action } => {
                    user_action.as_ref()?.downcast_ref().cloned()
                }
                navigation::PendingReleaseState::GloballyHeld { .. } => None,
                navigation::PendingReleaseState::GlobalHoldReleased { user_action } => {
                    user_action.downcast_ref().cloned()
                }
                navigation::PendingReleaseState::Invalidated { .. } => None,
            },
        }
    }
}
