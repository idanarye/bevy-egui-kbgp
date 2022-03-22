use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_egui::{EguiContext, EguiPlugin, EguiSettings};
use bevy_egui_kbgp::egui;
use bevy_egui_kbgp::prelude::*;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum MenuState {
    Main,
    Empty1,
    Empty2,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(KbgpPlugin)
        .insert_resource(EguiSettings { scale_factor: 2.0 })
        .insert_resource(KbgpSettings {
            allow_keyboard: true,
            allow_mouse_buttons: true,
            allow_mouse_wheel: true,
            allow_mouse_wheel_sideways: true,
            allow_gamepads: true,
            bindings: bevy_egui_kbgp::KbgpNavBindings::default()
                .with_key(KeyCode::W, KbgpNavAction::NavigateUp)
                .with_key(KeyCode::A, KbgpNavAction::NavigateLeft)
                .with_key(KeyCode::S, KbgpNavAction::NavigateDown)
                .with_key(KeyCode::D, KbgpNavAction::NavigateRight),
        })
        .add_state(MenuState::Main)
        .add_system_set(SystemSet::on_update(MenuState::Main).with_system(ui_system))
        .add_system_set(SystemSet::on_update(MenuState::Empty1).with_system(empty_state_system))
        .add_system_set(SystemSet::on_update(MenuState::Empty2).with_system(empty_state_system))
        .run();
}

fn menu_controls(ui: &mut egui::Ui, state: &mut State<MenuState>) {
    ui.horizontal(|ui| {
        let prev_state = match state.current() {
            MenuState::Main => MenuState::Empty2,
            MenuState::Empty1 => MenuState::Main,
            MenuState::Empty2 => MenuState::Empty1,
        };
        let next_state = match state.current() {
            MenuState::Main => MenuState::Empty1,
            MenuState::Empty1 => MenuState::Empty2,
            MenuState::Empty2 => MenuState::Main,
        };

        if ui
            .button(format!("<<{:?}<<", prev_state))
            .kbgp_navigation()
            .clicked()
        {
            state.set(prev_state).unwrap();
            ui.kbgp_clear_input();
        }

        ui.label(format!("{:?}", state.current()));

        if ui
            .button(format!(">>{:?}>>", next_state))
            .kbgp_navigation()
            .kbgp_initial_focus()
            .clicked()
        {
            state.set(next_state).unwrap();
            ui.ctx().kbgp_clear_input();
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State<MenuState>>,
    mut button_counters: Local<[usize; 4]>,
    mut checkbox_value: Local<bool>,
    mut label_value: Local<u8>,
    mut settable_inputs: Local<Vec<Option<KbgpInput>>>,
    mut settable_chords: Local<Vec<HashSet<KbgpInput>>>,
    mut settable_same_source_chords: Local<Vec<HashSet<KbgpInput>>>,
    gamepads: Res<Gamepads>,
    mut settable_inputs_of_source: Local<Vec<(KbgpInputSource, Vec<Option<KbgpInput>>)>>,
    mut settable_chords_of_source: Local<Vec<(KbgpInputSource, Vec<HashSet<KbgpInput>>)>>,
) {
    if settable_inputs.is_empty() {
        for _ in 0..3 {
            settable_inputs.push(None);
        }
    }
    for chords in [&mut *settable_chords, &mut *settable_same_source_chords] {
        if chords.is_empty() {
            for _ in 0..3 {
                chords.push(Default::default());
            }
        }
    }
    let mut all_input_sources = vec![KbgpInputSource::KeyboardAndMouse];
    all_input_sources.extend(gamepads.iter().copied().map(KbgpInputSource::Gamepad));
    while settable_inputs_of_source.len() < all_input_sources.len() {
        let source = all_input_sources[settable_inputs_of_source.len()];
        settable_inputs_of_source.push((source, vec![None; 3]));
    }
    while settable_chords_of_source.len() < all_input_sources.len() {
        let source = all_input_sources[settable_chords_of_source.len()];
        settable_chords_of_source.push((source, vec![Default::default(); 3]));
    }
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        menu_controls(ui, &mut state);
        ui.horizontal(|ui| {
            for counter in button_counters.iter_mut() {
                if ui
                    .button(format!("Counter: {counter}"))
                    .kbgp_navigation()
                    .clicked()
                {
                    *counter += 1;
                }
            }
        });
        if ui
            .checkbox(&mut checkbox_value.clone(), "Checkbox")
            .kbgp_navigation()
            .clicked()
        {
            *checkbox_value = !*checkbox_value;
        }
        ui.horizontal(|ui| {
            for i in 0..4 {
                if ui
                    .selectable_label(*label_value == i, format!("Value {i}"))
                    .kbgp_navigation()
                    .clicked()
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
        ui.horizontal(|ui| {
            ui.label("Set chord (same source):");
            for settable_chord in settable_same_source_chords.iter_mut() {
                let text = if settable_chord.is_empty() {
                    "N/A".to_owned()
                } else {
                    KbgpInput::format_chord(settable_chord.iter().cloned())
                };
                if let Some(chord) = ui
                    .button(text)
                    .kbgp_navigation()
                    .kbgp_pending_chord_same_source()
                {
                    *settable_chord = chord;
                }
            }
        });
        for (source, settable_inputs) in settable_inputs_of_source.iter_mut() {
            ui.horizontal(|ui| {
                ui.label(format!("Set key ({}):", source));
                for settable_input in settable_inputs.iter_mut() {
                    let text = if let Some(input) = settable_input {
                        format!("{}", input)
                    } else {
                        "N/A".to_owned()
                    };
                    if let Some(input) = ui
                        .button(text)
                        .kbgp_navigation()
                        .kbgp_pending_input_of_source(*source)
                    {
                        *settable_input = Some(input);
                    }
                }
            });
        }
        for (source, settable_chords) in settable_chords_of_source.iter_mut() {
            ui.horizontal(|ui| {
                ui.label(format!("Set chord ({}):", source));
                for settable_chord in settable_chords.iter_mut() {
                    let text = if settable_chord.is_empty() {
                        "N/A".to_owned()
                    } else {
                        KbgpInput::format_chord(settable_chord.iter().cloned())
                    };
                    if let Some(chord) = ui
                        .button(text)
                        .kbgp_navigation()
                        .kbgp_pending_chord_of_source(*source)
                    {
                        *settable_chord = chord;
                    }
                }
            });
        }
    });
}

fn empty_state_system(mut egui_context: ResMut<EguiContext>, mut state: ResMut<State<MenuState>>) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        menu_controls(ui, &mut state);
    });
}
