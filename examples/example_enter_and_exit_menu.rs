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
        .add_system(listen_to_menu_key.in_set(OnUpdate(AppState::NoMenu)))
        .add_system(ui_system.in_set(OnUpdate(AppState::Menu)))
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
    });
}
