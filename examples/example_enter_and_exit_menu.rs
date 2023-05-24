use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_egui_kbgp::prelude::*;
use bevy_egui_kbgp::{bevy_egui, egui};

#[derive(States, Default, Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum AppState {
    #[default]
    NoMenu,
    Menu,
}

#[derive(Clone, PartialEq, Eq)]
enum KbgpActions {
    ToggleMenu,
    ToggleMenuQ,
    ActionZ,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(EguiPlugin);
    app.add_plugin(KbgpPlugin);
    app.insert_resource(KbgpSettings {
        bindings: {
            bevy_egui_kbgp::KbgpNavBindings::default()
                .with_key(
                    KeyCode::Escape,
                    KbgpNavCommand::user(KbgpActions::ToggleMenu),
                )
                .with_key(KeyCode::Q, KbgpNavCommand::user(KbgpActions::ToggleMenuQ))
                .with_key(KeyCode::Z, KbgpNavCommand::user(KbgpActions::ActionZ))
        },
        ..Default::default()
    });
    app.add_state::<AppState>();
    app.insert_resource(ClickCountersForKeys(
        [
            KeyCode::Return,
            KeyCode::Space,
            KeyCode::Escape,
            KeyCode::Q,
            KeyCode::Z,
        ]
        .into_iter()
        .map(|key_code| (key_code, Default::default()))
        .collect(),
    ));
    app.add_system(listen_to_menu_key.in_set(OnUpdate(AppState::NoMenu)));
    app.add_system(ui_system.in_set(OnUpdate(AppState::Menu)));
    app.add_system(data_display_system);
    app.add_system(data_update_system.in_set(OnUpdate(AppState::NoMenu)));
    app.run();
}

fn listen_to_menu_key(mut egui_context: EguiContexts, mut state: ResMut<NextState<AppState>>) {
    let egui_context = egui_context.ctx_mut();
    if egui_context.kbgp_user_action() == Some(KbgpActions::ToggleMenu) {
        state.set(AppState::Menu);
        egui_context.kbgp_clear_input();
    }
    if egui_context.kbgp_user_action_released() == Some(KbgpActions::ToggleMenuQ) {
        state.set(AppState::Menu);
    }
}

fn ui_system(mut egui_context: EguiContexts, mut state: ResMut<NextState<AppState>>) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        ui.button("Does Nothing")
            .kbgp_navigation()
            .kbgp_initial_focus();
        if ui.kbgp_user_action() == Some(KbgpActions::ToggleMenu) {
            state.set(AppState::NoMenu);
            ui.kbgp_clear_input();
        }
        if ui.kbgp_user_action_released() == Some(KbgpActions::ToggleMenuQ) {
            state.set(AppState::NoMenu);
        }
        if ui
            .button("Exit Menu (immediately)")
            .kbgp_navigation()
            .clicked()
        {
            state.set(AppState::NoMenu);
            ui.kbgp_clear_input();
        }
        if ui
            .button("Exit Menu (after)")
            .kbgp_navigation()
            .kbgp_click_released()
        {
            state.set(AppState::NoMenu);
        }

        if ui
            .button("Exit Menu With Z (immediately)")
            .kbgp_navigation()
            .kbgp_user_action()
            == Some(KbgpActions::ActionZ)
        {
            state.set(AppState::NoMenu);
            ui.kbgp_clear_input();
        }
        if ui
            .button("Exit Menu With Z (after)")
            .kbgp_navigation()
            .kbgp_user_action_released()
            == Some(KbgpActions::ActionZ)
        {
            state.set(AppState::NoMenu);
        }
    });
}

#[derive(Default)]
struct ClickCounters {
    times_pressed: usize,
    times_released: usize,
    duration_pressed: Duration,
}

#[derive(Resource)]
struct ClickCountersForKeys(Vec<(KeyCode, ClickCounters)>);

fn data_display_system(
    mut egui_context: EguiContexts,
    click_counters_for_keys: Res<ClickCountersForKeys>,
) {
    let window = egui::Window::new("Clicks Data").default_pos([0.0, 200.0]);
    window.show(egui_context.ctx_mut(), |ui| {
        for (key_code, click_counters) in click_counters_for_keys.0.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", key_code));
                ui.vertical(|ui| {
                    ui.label(format!("Times Pressed: {}", click_counters.times_pressed));
                    ui.label(format!("Times Released: {}", click_counters.times_released));
                    ui.label(format!(
                        "Duration Pressed: {:.1?}",
                        click_counters.duration_pressed
                    ));
                });
            });
        }
    });
}

fn data_update_system(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut click_counters_for_keys: ResMut<ClickCountersForKeys>,
) {
    for (key_code, click_counters) in click_counters_for_keys.0.iter_mut() {
        if keyboard.just_pressed(*key_code) {
            click_counters.times_pressed += 1;
        }

        if keyboard.just_released(*key_code) {
            click_counters.times_released += 1;
        }

        if keyboard.pressed(*key_code) {
            click_counters.duration_pressed += time.delta();
        }
    }
}
