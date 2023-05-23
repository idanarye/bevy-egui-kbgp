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
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(KbgpPlugin)
        .insert_resource(KbgpSettings {
            bindings: {
                bevy_egui_kbgp::KbgpNavBindings::default().with_key(
                    KeyCode::Escape,
                    KbgpNavCommand::user(KbgpActions::ToggleMenu),
                )
            },
            ..Default::default()
        })
        .add_state::<AppState>()
        .init_resource::<ClickCounters>()
        .add_system(listen_to_menu_key.in_set(OnUpdate(AppState::NoMenu)))
        .add_system(ui_system.in_set(OnUpdate(AppState::Menu)))
        .add_system(data_display_system)
        .add_system(data_update_system.in_set(OnUpdate(AppState::NoMenu)))
        .run();
}

fn listen_to_menu_key(mut egui_context: EguiContexts, mut state: ResMut<NextState<AppState>>) {
    let egui_context = egui_context.ctx_mut();
    if egui_context.kbgp_user_action() == Some(KbgpActions::ToggleMenu) {
        state.set(AppState::Menu);
        egui_context.kbgp_clear_input();
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
        if ui.button("Exit Menu (immediately)").kbgp_navigation().clicked() {
            state.set(AppState::NoMenu);
            ui.kbgp_clear_input();
        }
    });
}

#[derive(Resource, Default)]
struct ClickCounters {
    times_pressed: usize,
    times_released: usize,
    duration_pressed: Duration,
}

fn data_display_system(mut egui_context: EguiContexts, click_counters: Res<ClickCounters>) {
    let window = egui::Window::new("Clicks Data").default_pos([0.0, 200.0]);
    window.show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("Times Pressed: {}", click_counters.times_pressed));
        ui.label(format!("Times Released: {}", click_counters.times_released));
        ui.label(format!(
            "Duration Pressed: {:.1?}",
            click_counters.duration_pressed
        ));
    });
}

fn data_update_system(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut click_counters: ResMut<ClickCounters>,
) {
    const KEYS: &[bevy::prelude::KeyCode] = &[KeyCode::Return, KeyCode::Space];

    if KEYS.iter().any(|key| keyboard.just_pressed(*key)) {
        click_counters.times_pressed += 1;
    }

    if KEYS.iter().any(|key| keyboard.just_released(*key)) {
        click_counters.times_released += 1;
    }

    if KEYS.iter().any(|key| keyboard.pressed(*key)) {
        click_counters.duration_pressed += time.delta();
    }
}
