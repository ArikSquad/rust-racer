use crate::scenes::GameScene;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::MainMenu), main_menu_setup)
            .add_systems(
                EguiPrimaryContextPass,
                main_menu_system.run_if(in_state(GameScene::MainMenu)),
            )
            .add_systems(OnExit(GameScene::MainMenu), main_menu_cleanup);
    }
}

fn main_menu_setup() {}
fn main_menu_cleanup() {}

fn main_menu_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameScene>>,
    mut exit: MessageWriter<AppExit>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(120.0);
            ui.label(egui::RichText::new("Unnamed Project").size(40.0));
            ui.add_space(40.0);
            if ui.button("New Game").clicked() {
                next_state.set(GameScene::Prologue);
            }
            if ui.button("Options").clicked() {
                next_state.set(GameScene::Options);
            }
            if ui.button("Quit").clicked() {
                exit.write(AppExit::Success);
            }
        });
    });
}
