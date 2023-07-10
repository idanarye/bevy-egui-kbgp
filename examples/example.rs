use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_egui::{EguiContexts, EguiPlugin, EguiSettings};
use bevy_egui_kbgp::egui;
use bevy_egui_kbgp::prelude::*;

#[derive(States, Default, Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum MenuState {
    #[default]
    Main,
    Empty1,
    Empty2,
}

#[derive(Clone, PartialEq, Eq)]
enum MyActions {
    PrevMenu,
    NextMenu,
    Delete,
}

#[derive(PartialEq)]
enum MyFocusLabel {
    PrevMenu,
    NextMenu,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_plugins(KbgpPlugin);
    app.insert_resource(EguiSettings {
        scale_factor: 1.5,
        ..Default::default()
    });
    app.insert_resource(KbgpSettings {
        disable_default_navigation: true,
        disable_default_activation: true,
        prevent_loss_of_focus: true,
        focus_on_mouse_movement: true,
        allow_keyboard: true,
        allow_mouse_buttons: true,
        allow_mouse_wheel: true,
        allow_mouse_wheel_sideways: true,
        allow_gamepads: true,
        bindings: {
            bevy_egui_kbgp::KbgpNavBindings::default()
                .with_wasd_navigation()
                .with_key(KeyCode::Space, KbgpNavCommand::Click)
                // Special actions - keyboard:
                .with_key(KeyCode::PageUp, KbgpNavCommand::user(MyActions::PrevMenu))
                .with_key(KeyCode::PageDown, KbgpNavCommand::user(MyActions::NextMenu))
                .with_key(KeyCode::Delete, KbgpNavCommand::user(MyActions::Delete))
                // Special actions - gamepad:
                .with_gamepad_button(
                    GamepadButtonType::LeftTrigger,
                    KbgpNavCommand::user(MyActions::PrevMenu),
                )
                .with_gamepad_button(
                    GamepadButtonType::RightTrigger,
                    KbgpNavCommand::user(MyActions::NextMenu),
                )
                .with_gamepad_button(
                    GamepadButtonType::North,
                    KbgpNavCommand::user(MyActions::Delete),
                )
        },
    });
    app.add_state::<MenuState>();
    app.add_systems(Update, ui_system.run_if(in_state(MenuState::Main)));
    app.add_systems(Update, empty_state_system.run_if(in_state(MenuState::Empty1)));
    app.add_systems(Update, empty_state_system.run_if(in_state(MenuState::Empty2)));
    app.run();
}

fn menu_controls(
    ui: &mut egui::Ui,
    state: &State<MenuState>,
    next_state_setter: &mut NextState<MenuState>,
) {
    ui.label("Navigation: arrow keys, WASD, gamepad's d-pad, gamepad's left stick.");
    ui.label("Primary action: left-click, Enter, Space, gamepad's south button.");
    ui.label("Secondary action: right-click, Delete key, gamepad's north button.");
    ui.label("Change menu: page up/down, gamepad's upper triggers, these buttons here:");
    ui.horizontal(|ui| {
        let prev_state = match state.get() {
            MenuState::Main => MenuState::Empty2,
            MenuState::Empty1 => MenuState::Main,
            MenuState::Empty2 => MenuState::Empty1,
        };
        let next_state = match state.get() {
            MenuState::Main => MenuState::Empty1,
            MenuState::Empty1 => MenuState::Empty2,
            MenuState::Empty2 => MenuState::Main,
        };

        if ui
            .button(format!("<<{:?}<<", prev_state))
            .kbgp_navigation()
            .kbgp_focus_label(MyFocusLabel::PrevMenu)
            .clicked()
            || ui.kbgp_user_action() == Some(MyActions::PrevMenu)
        {
            next_state_setter.set(prev_state);
            ui.kbgp_clear_input();
            ui.kbgp_set_focus_label(MyFocusLabel::PrevMenu);
        }

        ui.label(format!("{:?}", state.get()));

        if ui
            .button(format!(">>{:?}>>", next_state))
            .kbgp_navigation()
            .kbgp_initial_focus()
            .kbgp_focus_label(MyFocusLabel::NextMenu)
            .clicked()
            || ui.kbgp_user_action() == Some(MyActions::NextMenu)
        {
            next_state_setter.set(next_state);
            ui.ctx().kbgp_clear_input();
            ui.ctx().kbgp_set_focus_label(MyFocusLabel::NextMenu);
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn ui_system(
    mut egui_context: EguiContexts,
    state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut button_counters: Local<[usize; 4]>,
    mut counter_on_release: Local<usize>,
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
    all_input_sources.extend(gamepads.iter().map(KbgpInputSource::Gamepad));
    while settable_inputs_of_source.len() < all_input_sources.len() {
        let source = all_input_sources[settable_inputs_of_source.len()];
        settable_inputs_of_source.push((source, vec![None; 3]));
    }
    while settable_chords_of_source.len() < all_input_sources.len() {
        let source = all_input_sources[settable_chords_of_source.len()];
        settable_chords_of_source.push((source, vec![Default::default(); 3]));
    }
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        menu_controls(ui, &state, &mut next_state);
        ui.horizontal(|ui| {
            for counter in button_counters.iter_mut() {
                match ui
                    .button(format!("Counter: {counter}"))
                    .kbgp_navigation()
                    .kbgp_activated()
                {
                    KbgpNavActivation::Clicked => {
                        *counter += 1;
                    }
                    KbgpNavActivation::ClickedSecondary
                    | KbgpNavActivation::User(MyActions::Delete) => {
                        if 0 < *counter {
                            *counter -= 1;
                        }
                    }
                    _ => {}
                }
            }
        });
        match ui
            .button(format!(
                "Counter (activates on key release): {}",
                *counter_on_release
            ))
            .kbgp_navigation()
            .kbgp_activate_released()
        {
            KbgpNavActivation::Clicked => {
                *counter_on_release += 1;
            }
            KbgpNavActivation::ClickedSecondary | KbgpNavActivation::User(MyActions::Delete) => {
                if 0 < *counter_on_release {
                    *counter_on_release -= 1;
                }
            }
            _ => {}
        }
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

        fn check_for_delete_action(button: &egui::Response) -> bool {
            matches!(
                button.kbgp_activated(),
                KbgpNavActivation::ClickedSecondary | KbgpNavActivation::User(MyActions::Delete)
            )
        }

        ui.horizontal(|ui| {
            ui.label("Set key:");
            for settable_input in settable_inputs.iter_mut() {
                let text = if let Some(input) = settable_input {
                    format!("{}", input)
                } else {
                    "N/A".to_owned()
                };
                let button = ui.button(text).kbgp_navigation();
                if let Some(input) = button.kbgp_pending_input() {
                    *settable_input = Some(input);
                } else if check_for_delete_action(&button) {
                    *settable_input = None;
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
                let button = ui.button(text).kbgp_navigation();
                if let Some(chord) = button.kbgp_pending_chord() {
                    *settable_chord = chord;
                } else if check_for_delete_action(&button) {
                    *settable_chord = Default::default();
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
                let button = ui.button(text).kbgp_navigation();
                if let Some(chord) = button.kbgp_pending_chord_same_source() {
                    *settable_chord = chord;
                } else if check_for_delete_action(&button) {
                    *settable_chord = Default::default();
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
                    let button = ui.button(text).kbgp_navigation();
                    if let Some(input) = button.kbgp_pending_input_of_source(*source) {
                        *settable_input = Some(input);
                    } else if check_for_delete_action(&button) {
                        *settable_input = None;
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
                    let button = ui.button(text).kbgp_navigation();
                    if let Some(chord) = button.kbgp_pending_chord_of_source(*source) {
                        *settable_chord = chord;
                    } else if check_for_delete_action(&button) {
                        *settable_chord = Default::default();
                    }
                }
            });
        }

        ui.horizontal(|ui| {
            #[derive(PartialEq)]
            enum FocusLabel {
                Left,
                Right,
            }
            if ui
                .button("Focus >")
                .kbgp_navigation()
                .kbgp_focus_label(FocusLabel::Left)
                .clicked()
            {
                ui.kbgp_set_focus_label(FocusLabel::Right);
            }
            if ui
                .button("< Focus")
                .kbgp_navigation()
                .kbgp_focus_label(FocusLabel::Right)
                .clicked()
            {
                ui.kbgp_set_focus_label(FocusLabel::Left);
            }
        });
    });
}

fn empty_state_system(
    mut egui_context: EguiContexts,
    state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        menu_controls(ui, &state, &mut next_state);
    });
}
