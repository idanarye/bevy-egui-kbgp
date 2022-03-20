//! Improve the keyboard and gamepads usage for egui in Bevy.
//!
//! Usage:
//! * Add [`KbgpPlugin`](crate::KbgpPlugin).
//! * Use [the extension methods](crate::KbgpEguiResponseExt) on the egui widgets to add KBGP's
//!   functionality.
//!
//! ```no_run
//! use bevy_egui_kbgp::{egui, bevy_egui};
//! use bevy::prelude::*;
//! use bevy_egui::{EguiContext, EguiPlugin};
//! use bevy_egui_kbgp::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(EguiPlugin)
//!         .add_plugin(KbgpPlugin)
//!         .add_system(ui_system)
//!         .run();
//! }
//!
//! fn ui_system(
//!     mut egui_context: ResMut<EguiContext>,
//!     keys: Res<Input<KeyCode>>,
//! ) {
//!     egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
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
//! }
//! ```

pub use bevy_egui;
pub use bevy_egui::egui;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_egui::EguiContext;

use self::navigation::KbgpNavigationState;
pub use self::navigation::KbgpPrepareNavigation;
use self::pending_input::KbgpPendingInputState;
pub use self::pending_input::{KbgpInputManualHandle, KbgpPreparePendingInput};

mod navigation;
mod pending_input;

pub mod prelude {
    pub use crate::KbgpPlugin;
    pub use crate::KbgpSettings;
    pub use crate::kbgp_prepare;
    pub use crate::KbgpEguiResponseExt;
    pub use crate::KbgpInput;
}

/// Adds KBGP input handling system and [`KbgpSettings`](crate::KbgpSettings).
pub struct KbgpPlugin;

impl Plugin for KbgpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KbgpSettings::default());
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            kbgp_system_default_input.after(bevy_egui::EguiSystem::ProcessInput),
        );
    }
}

/// General configuration resource for KBGP.
///
/// Note: [`KbgpPlugin`](crate::KbgpPlugin) will add the default settings, so custom settings
/// should either be added after the plugin or modified with a system.
pub struct KbgpSettings {
    /// Whether or not gamepads input is accepted for navigation and for chords.
    pub allow_gamepads: bool,
}

impl Default for KbgpSettings {
    fn default() -> Self {
        Self {
            allow_gamepads: true,
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
    egui_ctx
        .memory()
        .data
        .get_temp_mut_or_default::<std::sync::Arc<egui::mutex::Mutex<Kbgp>>>(egui::Id::null())
        .clone()
}

/// Must be called every frame, either manually or by using [`KbgpPlugin`].
///
/// Should be called between bevy_egui's input handling system and the system that generates the
/// UI - so in the `CoreStage::PreUpdate` stage after the `EguiSystem::ProcessInput` label.
///
/// The `prepare_dlg` argument is a closure that accepts a [`KbgpPrepare`](crate::KbgpPrepare), and
/// used to:
///
/// * Register the input from the keyboard and the gamepads.
/// * Set preferences.
///
/// Typical usage:
///
/// ```no_run
/// # use bevy_egui_kbgp::bevy_egui;
/// # use bevy::prelude::*;
/// # use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
/// # use bevy_egui_kbgp::prelude::*;
/// # use bevy_egui_kbgp::KbgpPrepare;
/// fn custom_kbgp_system(
///     mut egui_context: ResMut<EguiContext>,
///     keys: Res<Input<KeyCode>>,
///     gamepads: Res<Gamepads>,
///     gamepad_axes: Res<Axis<GamepadAxis>>,
///     gamepad_buttons: Res<Input<GamepadButton>>,
/// ) {
///     kbgp_prepare(egui_context.ctx_mut(), |prp| {
///         match prp {
///             KbgpPrepare::Navigation(prp) => {
///                 prp.navigate_keyboard_default(&keys);
///                 prp.navigate_gamepad_default(&gamepads, &gamepad_axes, &gamepad_buttons);
///             }
///             KbgpPrepare::PendingInput(prp) => {
///                 prp.accept_keyboard_input(&keys);
///                 prp.accept_gamepad_input(&gamepads, &gamepad_axes, &gamepad_buttons);
///             }
///         }
///     });
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
        KbgpState::Inactive => {
            if !kbgp.common.nodes.is_empty() {
                kbgp.state = KbgpState::Navigation(KbgpNavigationState::default());
            }
        }
        KbgpState::Navigation(state) => {
            state.prepare(common, egui_ctx, |prp| {
                prepare_dlg(KbgpPrepare::Navigation(prp))
            });
        }
        KbgpState::PendingInput(state) => {
            state.prepare(common, egui_ctx, |prp| {
                prepare_dlg(KbgpPrepare::PendingInput(prp))
            });
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
fn kbgp_system_default_input(
    mut egui_context: ResMut<EguiContext>,
    settings: Res<KbgpSettings>,
    keys: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
) {
    kbgp_prepare(egui_context.ctx_mut(), |prp| {
        match prp {
            KbgpPrepare::Navigation(prp) => {
                prp.navigate_keyboard_default(&keys);
                if settings.allow_gamepads {
                    prp.navigate_gamepad_default(&gamepads, &gamepad_axes, &gamepad_buttons);
                }
            }
            KbgpPrepare::PendingInput(prp) => {
                prp.accept_keyboard_input(&keys);
                if settings.allow_gamepads {
                    prp.accept_gamepad_input(&gamepads, &gamepad_axes, &gamepad_buttons);
                }
            }
        }
    });
}

#[derive(Default)]
struct KbgpCommon {
    nodes: HashMap<egui::Id, NodeData>,
}

enum KbgpState {
    Inactive,
    Navigation(KbgpNavigationState),
    PendingInput(KbgpPendingInputState),
}

impl Default for KbgpState {
    fn default() -> Self {
        Self::Inactive
    }
}

#[derive(Debug)]
struct NodeData {
    rect: egui::Rect,
    seen_this_frame: bool,
}

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
pub trait KbgpEguiResponseExt {
    /// When the UI is first created, focus on this widget.
    fn kbgp_initial_focus(self) -> Self;

    /// Navigate to and from this widget.
    fn kbgp_navigation(self) -> Self;

    /// Accept a single key/button input from this widget.
    ///
    /// Must be called on widgets that had
    /// [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) called on them.
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_egui::{EguiContext, EguiPlugin};
    /// use bevy_egui_kbgp::{egui, bevy_egui};
    /// use bevy_egui_kbgp::prelude::*;
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugin(EguiPlugin)
    ///         .add_plugin(KbgpPlugin)
    ///         .add_system(ui_system)
    ///         .insert_resource(JumpInput(KbgpInput::Keyboard(KeyCode::Space)))
    ///         .run();
    /// }
    ///
    /// struct JumpInput(KbgpInput);
    ///
    /// fn ui_system(
    ///     mut egui_context: ResMut<EguiContext>,
    ///     mut jump_input: ResMut<JumpInput>,
    /// ) {
    ///     egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
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
    /// }
    fn kbgp_pending_input(&self) -> Option<KbgpInput>;

    fn kbgp_pending_input_of_gamepad(&self, gamepad: Option<Gamepad>) -> Option<KbgpInput>;

    /// Accept a chord of key/button inputs from this widget.
    ///
    /// Must be called on widgets that had
    /// [`kbgp_navigation`](crate::KbgpEguiResponseExt::kbgp_navigation) called on them.
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_egui::{EguiContext, EguiPlugin};
    /// use bevy_egui_kbgp::{egui, bevy_egui};
    /// use bevy_egui_kbgp::prelude::*;
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugin(EguiPlugin)
    ///         .add_plugin(KbgpPlugin)
    ///         .add_system(ui_system)
    ///         .insert_resource(JumpChord(vec![KbgpInput::Keyboard(KeyCode::Space)]))
    ///         .run();
    /// }
    ///
    /// struct JumpChord(Vec<KbgpInput>);
    ///
    /// fn ui_system(
    ///     mut egui_context: ResMut<EguiContext>,
    ///     mut jump_chord: ResMut<JumpChord>,
    /// ) {
    ///     egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
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
    /// }
    fn kbgp_pending_chord(&self) -> Option<HashSet<KbgpInput>>;

    fn kbgp_pending_chord_of_gamepad(&self, gamepad: Option<Gamepad>)
        -> Option<HashSet<KbgpInput>>;

    fn kbgp_pending_chord_same_source(&self) -> Option<HashSet<KbgpInput>>;

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
    fn kbgp_initial_focus(self) -> Self {
        let kbgp = kbgp_get(&self.ctx);
        let kbgp = kbgp.lock();
        match kbgp.state {
            KbgpState::Inactive => {
                self.request_focus();
            }
            KbgpState::Navigation(_) => {}
            KbgpState::PendingInput(_) => {}
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
                seen_this_frame: true,
            },
        );
        match &kbgp.state {
            KbgpState::Inactive => {}
            KbgpState::Navigation(state) => {
                if Some(self.id) == state.move_focus || self.clicked() {
                    self.request_focus();
                }
            }
            KbgpState::PendingInput(_) => {}
        }
        self
    }

    fn kbgp_pending_input_manual<T>(
        &self,
        dlg: impl FnOnce(&Self, KbgpInputManualHandle) -> Option<T>,
    ) -> Option<T> {
        let kbgp = kbgp_get(&self.ctx);
        let mut kbgp = kbgp.lock();
        match &mut kbgp.state {
            KbgpState::Inactive => None,
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
                self.ctx.memory().lock_focus(self.id, true);
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
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|hnd, _| hnd.received_input().is_empty());
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

    fn kbgp_pending_input_of_gamepad(&self, gamepad: Option<Gamepad>) -> Option<KbgpInput> {
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|hnd, input| {
                input.get_gamepad() == gamepad && hnd.received_input().is_empty()
            });
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
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|_, _| true);
            hnd.show_current_chord(response);
            if hnd.input_this_frame().any(|_| true) || hnd.received_input().is_empty() {
                None
            } else {
                Some(hnd.received_input().clone())
            }
        })
    }

    fn kbgp_pending_chord_of_gamepad(
        &self,
        gamepad: Option<Gamepad>,
    ) -> Option<HashSet<KbgpInput>> {
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|_, input| input.get_gamepad() == gamepad);
            hnd.show_current_chord(response);
            if hnd.input_this_frame().any(|_| true) || hnd.received_input().is_empty() {
                None
            } else {
                Some(hnd.received_input().clone())
            }
        })
    }

    fn kbgp_pending_chord_same_source(&self) -> Option<HashSet<KbgpInput>> {
        self.kbgp_pending_input_manual(|response, mut hnd| {
            hnd.process_new_input(|hnd, input| {
                if let Some(existing_input) = hnd.received_input().iter().next() {
                    input.get_gamepad() == existing_input.get_gamepad()
                } else {
                    true
                }
            });
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
    GamepadAxisPositive(GamepadAxis),
    GamepadAxisNegative(GamepadAxis),
    GamepadButton(GamepadButton),
}

impl core::fmt::Display for KbgpInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KbgpInput::Keyboard(key) => write!(f, "{:?}", key)?,
            KbgpInput::GamepadButton(GamepadButton(Gamepad(gamepad), button)) => {
                write!(f, "[{}]{:?}", gamepad, button)?
            }
            KbgpInput::GamepadAxisPositive(GamepadAxis(Gamepad(gamepad), axis)) => {
                write!(f, "[{}]{:?}", gamepad, axis)?
            }
            KbgpInput::GamepadAxisNegative(GamepadAxis(Gamepad(gamepad), axis)) => {
                write!(f, "[{}]-{:?}", gamepad, axis)?
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
            write!(&mut chord_text, "{}", input).unwrap();
        }
        chord_text
    }

    pub fn get_gamepad(&self) -> Option<Gamepad> {
        match self {
            KbgpInput::Keyboard(_) => None,
            KbgpInput::GamepadAxisPositive(GamepadAxis(gamepad, _)) => Some(*gamepad),
            KbgpInput::GamepadAxisNegative(GamepadAxis(gamepad, _)) => Some(*gamepad),
            KbgpInput::GamepadButton(GamepadButton(gamepad, _)) => Some(*gamepad),
        }
    }
}
