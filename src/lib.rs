//! Improve the keyboard and gamepads usage for egui in Bevy.
//!
//! Place a [`Kbgp`](crate::Kbgp) from inside a `Local` resource to maintain state. Call its
//! [`prepare`](crate::Kbgp::prepare) each frame to pass the input (and possibly set parameters).
//! Use [the extension methods](crate::KbgpEguiResponseExt) on the egui widgets to add KBGP's
//! functionality.
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
//! # use bevy_egui_kbgp::prelude::*;
//! fn ui_system(
//!     mut egui_context: ResMut<EguiContext>,
//!     keys: Res<Input<KeyCode>>,
//! ) {
//!     egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
//!         if ui
//!             .button("First Button")
//!             .kbgp_initial_focus()
//!             .kbgp_navigation()
//!             .kbgp_activated()
//!         {
//!             // First button action
//!         }
//!
//!         if ui
//!             .button("Second Button")
//!             .kbgp_navigation()
//!             .kbgp_activated()
//!         {
//!             // Second button action
//!         }
//!     });
//! }
//! ```

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_egui::EguiContext;

use self::navigation::KbgpNavigationState;

mod navigation;

pub mod prelude {
    pub use crate::kbgp_prepare;
    pub use crate::kbgp_system_default_input;
    pub use crate::navigation::KbgpEguiResponseExt;
}

/// Object used to configure KBGP's behavior in [`Kbgp::prepare`](crate::Kbgp::prepare).
pub struct KbgpPrepareHandle {
    /// When the user holds a key/button, KBGP will wait `secs_after_first_input` seconds before
    /// starting to rapidly apply the action.
    ///
    /// Default: 0.6 seconds.
    pub secs_after_first_input: f64,
    /// When the user holds a key/button, after
    /// [`secs_after_first_input`](crate::KbgpPrepareHandle::secs_after_first_input), KBGP
    /// will apply the action every `secs_between_inputs` seconds.
    ///
    /// Default: 0.04 seconds.
    pub secs_between_inputs: f64,
    pub(crate) input: u8,
}

/// KBGP's state holder:
///
/// * Must be preserved between frames - usually with a `Local` resource.
/// * [`prepare`](crate::Kbgp::prepare) must be called each frame, before drawing the GUI.
/// * Must be passed as a reference to [the `Response` extension methods](crate::KbgpEguiResponseExt).
#[derive(Default)]
pub struct Kbgp {
    common: KbgpCommon,
    state: KbgpState,
}

fn kbgp_get(egui_ctx: &egui::CtxRef) -> std::sync::Arc<egui::mutex::Mutex<Kbgp>> {
    egui_ctx
        .memory()
        .data
        .get_temp_mut_or_default::<std::sync::Arc<egui::mutex::Mutex<Kbgp>>>(egui::Id::null())
        .clone()
}

/// Must be called every frame, before drawing.
///
/// The `prepare_dlg` argument is a closure that accepts a
/// [`KbgpPrepareHandle`](crate::KbgpPrepareHandle), and used to:
///
/// * Register the input from the keyboard and the gamepads.
/// * Set preferences.
///
/// Typical usage:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
/// # use bevy_egui_kbgp::prelude::*;
/// fn custom_kbgp_system(
///     mut egui_context: ResMut<EguiContext>,
///     keys: Res<Input<KeyCode>>,
///     gamepads: Res<Gamepads>,
///     gamepad_axes: Res<Axis<GamepadAxis>>,
///     gamepad_buttons: Res<Input<GamepadButton>>,
/// ) {
///     kbgp_prepare(egui_context.ctx_mut(), |prp| {
///         prp.navigate_keyboard_default(&keys);
///         prp.navigate_gamepad_default(&gamepads, &gamepad_axes, &gamepad_buttons);
///     });
/// }
/// ```
pub fn kbgp_prepare(egui_ctx: &egui::CtxRef, prepare_dlg: impl FnOnce(&mut KbgpPrepareHandle)) {
    let kbgp = kbgp_get(egui_ctx);
    let mut kbgp = kbgp.lock();
    kbgp.common.nodes.retain(|_, data| data.still_there);
    for node_data in kbgp.common.nodes.values_mut() {
        node_data.still_there = false;
    }
    let Kbgp { common, state } = &mut *kbgp;
    match state {
        KbgpState::Inactive => {
            if !kbgp.common.nodes.is_empty() {
                kbgp.state = KbgpState::Navigation(KbgpNavigationState::default());
            }
        }
        KbgpState::Navigation(state) => {
            state.prepare(common, egui_ctx, prepare_dlg);
        }
    }
}

pub fn kbgp_system_default_input(
    mut egui_context: ResMut<EguiContext>,
    keys: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
) {
    kbgp_prepare(egui_context.ctx_mut(), |prp| {
        prp.navigate_keyboard_default(&keys);
        prp.navigate_gamepad_default(&gamepads, &gamepad_axes, &gamepad_buttons);
    });
}

#[derive(Default)]
struct KbgpCommon {
    nodes: HashMap<egui::Id, NodeData>,
}

enum KbgpState {
    Inactive,
    Navigation(KbgpNavigationState),
}

impl Default for KbgpState {
    fn default() -> Self {
        Self::Inactive
    }
}

#[derive(Debug)]
struct NodeData {
    rect: egui::Rect,
    still_there: bool,
}
