use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_egui_kbgp::prelude::*;
use bevy_egui_kbgp::{bevy_egui, egui};

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum AppState {
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
        .add_state(AppState::NoMenu)
        .add_system_set(SystemSet::on_update(AppState::NoMenu).with_system(listen_to_menu_key))
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(ui_system))
        .run();
}

fn listen_to_menu_key(mut state: ResMut<State<AppState>>, mut egui_context: ResMut<EguiContext>) {
    let egui_context = egui_context.ctx_mut();
    if egui_context.kbgp_user_action() == Some(KbgpActions::ToggleMenu) {
        state.set(AppState::Menu).unwrap();
        egui_context.kbgp_clear_input();
    }
}

fn ui_system(mut egui_context: ResMut<EguiContext>, mut state: ResMut<State<AppState>>) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        ui.button("Does Nothing")
            .kbgp_navigation()
            .kbgp_initial_focus();
        if ui.kbgp_user_action() == Some(KbgpActions::ToggleMenu) {
            state.set(AppState::NoMenu).unwrap();
            ui.kbgp_clear_input();
        }
    });
}
