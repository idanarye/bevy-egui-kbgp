use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiContext, EguiSettings};
use bevy_egui_kbgp::{Kbgp, KbgpEguiResponseExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(EguiSettings {
            scale_factor: 5.0,
        })
        .add_system(ui_system)
        .run();
}

fn ui_system(mut egui_context: ResMut<EguiContext>, mut kbgp: Local<Kbgp>, keys: Res<Input<KeyCode>>) {
    kbgp.prepare(egui_context.ctx_mut(), |prp| {
        prp.navigate_keyboard_default(&keys);
    });
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            for col in ['a', 'b', 'c'] {
                ui.vertical(|ui| {
                    for row in 1..10 {
                        if ui.button(format!("button {col}{row}")).kbgp_navigation(&mut kbgp).clicked() {
                            println!("Clicked button {col}{row}");
                        }
                    }
                });
            }
        });
    });
}
