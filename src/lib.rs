//! Improve the keyboard and gamepads usage for egui in Bevy.
//!
//! Usage:
//! * Either use the [`kbgp_system_default_input`](crate::kbgp_system_default_input) system or
//!   call [`kbgp_prepare`](crate::kbgp_prepare) with custom inputs and/or with a non-default egui
//!   context.
//! * Use [the extension methods](crate::KbgpEguiResponseExt) on the egui widgets to add KBGP's
//!   functionality.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_egui::{EguiContext, EguiPlugin};
//! use bevy_egui_kbgp::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(EguiPlugin)
//!         .add_system(kbgp_system_default_input)
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
pub use self::navigation::KbgpPrepareNavigation;

mod navigation;

pub mod prelude {
    pub use crate::kbgp_prepare;
    pub use crate::kbgp_system_default_input;
    pub use crate::KbgpEguiResponseExt;
}

/// Object used to configure KBGP's behavior in [`kbgp_prepare`].
pub enum KbgpPrepare<'a> {
    Navigation(&'a mut KbgpPrepareNavigation),
}

impl KbgpPrepare<'_> {
    /// Apply the default KBGP input scheme.
    ///
    /// The [`kbgp_system_default_input`](crate::kbgp_system_default_input) system already applies
    /// this to the default egui context, so it is preferrable to use that, but in case it needs to
    /// be applied to a different egui context - this method can be used instead:
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_egui_kbgp::prelude::*;
    /// # use bevy_egui::EguiContext;
    /// # let window_id = bevy::window::WindowId::new();
    /// # let mut egui_context: ResMut<EguiContext> = panic!();
    /// # let keys: Res<Input<KeyCode>> = panic!();
    /// # let gamepads: Res<Gamepads> = panic!();
    /// # let gamepad_axes: Res<Axis<GamepadAxis>> = panic!();
    /// # let gamepad_buttons: Res<Input<GamepadButton>> = panic!();
    /// kbgp_prepare(egui_context.ctx_for_window_mut(window_id), |mut prp| {
    ///     prp.default_input(&keys, &gamepads, &gamepad_axes, &gamepad_buttons);
    /// });
    /// ```
    pub fn default_input(
        &mut self,
        keys: &Input<KeyCode>,
        gamepads: &Gamepads,
        gamepad_axes: &Axis<GamepadAxis>,
        gamepad_buttons: &Input<GamepadButton>,
    ) {
        match self {
            KbgpPrepare::Navigation(prp) => {
                prp.navigate_keyboard_default(&keys);
                prp.navigate_gamepad_default(&gamepads, &gamepad_axes, &gamepad_buttons);
            }
        }
    }
}

#[derive(Default)]
struct Kbgp {
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

/// Must be called every frame, either manually or by using [`kbgp_system_default_input`].
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
///         }
///     });
/// }
/// ```
pub fn kbgp_prepare(egui_ctx: &egui::CtxRef, prepare_dlg: impl FnOnce(KbgpPrepare<'_>)) {
    let kbgp = kbgp_get(egui_ctx);
    let mut kbgp = kbgp.lock();
    // Since Bevy is allow to reorder systems mid-run, there is a risk that the KBGP prepare system
    // run twice between egui drawing systems. The stale counter allows up to two such invocations
    // - after that it assumes the widget is no longer drawn.
    kbgp.common.nodes.retain(|_, data| data.stale_counter < 2);
    for node_data in kbgp.common.nodes.values_mut() {
        node_data.stale_counter += 1;
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
pub fn kbgp_system_default_input(
    mut egui_context: ResMut<EguiContext>,
    keys: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
) {
    kbgp_prepare(egui_context.ctx_mut(), |mut prp| {
        prp.default_input(&keys, &gamepads, &gamepad_axes, &gamepad_buttons);
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
    stale_counter: u8,
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
