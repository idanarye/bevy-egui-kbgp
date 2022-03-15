use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
use bevy_egui_kbgp::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(EguiSettings { scale_factor: 2.0 })
        .add_system(kbgp_system_default_input)
        .add_system(ui_system)
        .run();
}

#[allow(clippy::too_many_arguments)]
fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut button_counters: Local<[usize; 4]>,
    mut checkbox_value: Local<bool>,
    mut label_value: Local<u8>,
) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        ui.button("Holds focus on startup")
            .kbgp_initial_focus()
            .kbgp_navigation();
        ui.horizontal(|ui| {
            for counter in button_counters.iter_mut() {
                if ui
                    .button(format!("Counter: {counter}"))
                    .kbgp_navigation()
                    .kbgp_activated()
                {
                    *counter += 1;
                }
            }
        });
        if ui
            .checkbox(&mut checkbox_value.clone(), "Checkbox")
            .kbgp_navigation()
            .kbgp_activated()
        {
            *checkbox_value = !*checkbox_value;
        }
        ui.horizontal(|ui| {
            for i in 0..4 {
                if ui
                    .selectable_label(*label_value == i, format!("Value {i}"))
                    .kbgp_navigation()
                    .kbgp_activated()
                {
                    *label_value = i;
                }
            }
        });
    });
}
