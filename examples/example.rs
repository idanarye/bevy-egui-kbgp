use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
use bevy_egui_kbgp::egui;
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
    mut settable_inputs: Local<Vec<Option<KbgpInput>>>,
    mut settable_chords: Local<Vec<HashSet<KbgpInput>>>,
) {
    if settable_inputs.is_empty() {
        for _ in 0..3 {
            settable_inputs.push(None);
        }
    }
    if settable_chords.is_empty() {
        for _ in 0..3 {
            settable_chords.push(Default::default());
        }
    }
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        ui.button("This button doesn't do anything - it only demonstrates focus grabbing when the GUI is created")
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
        ui.horizontal(|ui| {
            ui.label("Set key:");
            for settable_input in settable_inputs.iter_mut() {
                let text = if let Some(input) = settable_input {
                    format!("{}", input)
                } else {
                    "N/A".to_owned()
                };
                if let Some(input) = ui.button(text).kbgp_navigation().kbgp_pending_input() {
                    *settable_input = Some(input);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Set chord:");
            for settable_chord in settable_chords.iter_mut() {
                let text = if settable_chord.is_empty() {
                    "N/A".to_owned()
                } else {
                    KbgpInput::format_chord(settable_chord.iter().cloned())
                };
                if let Some(chord) = ui.button(text).kbgp_navigation().kbgp_pending_chord() {
                    *settable_chord = chord;
                }
            }
        });
    });
}
