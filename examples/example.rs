use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
use bevy_egui_kbgp::{Kbgp, KbgpEguiResponseExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(EguiSettings { scale_factor: 2.0 })
        .add_system(ui_system)
        .run();
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut kbgp: Local<Kbgp>,
    keys: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
) {
    kbgp.prepare(egui_context.ctx_mut(), |prp| {
        prp.navigate_keyboard_default(&keys);
        prp.navigate_gamepad_default(&gamepads, &gamepad_axes, &gamepad_buttons);
    });
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            for col in ['a', 'b', 'c'] {
                ui.vertical(|ui| {
                    for row in 1..10 {
                        if ui
                            .button(format!("button {col}{row}"))
                            .kbgp_navigation(&mut kbgp)
                            .kbgp_activated(&kbgp)
                        {
                            println!("Clicked button {col}{row}");
                        }
                    }
                });
            }
        });
    });
}
