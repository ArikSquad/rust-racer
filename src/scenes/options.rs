use bevy::prelude::*;
use crate::scenes::GameScene;
use bevy_egui::{EguiContexts, egui, EguiPrimaryContextPass};

pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::Options), options_setup)
            .add_systems(EguiPrimaryContextPass, options_system.run_if(in_state(GameScene::Options)))
            .add_systems(OnExit(GameScene::Options), options_cleanup);
    }
}

#[derive(Resource, Default)]
pub struct OptionsState {
    pub renderer: String,
    pub vsync: bool,
    pub fps_cap: u32,
}

fn options_setup(mut commands: Commands) {
    commands.insert_resource(OptionsState {
        renderer: "vulkan".to_string(),
        vsync: true,
        fps_cap: 60,
    });
}

fn options_system(
    mut contexts: EguiContexts,
    mut state: ResMut<OptionsState>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label(egui::RichText::new("Options").size(36.0));
            ui.add_space(20.0);
            egui::ComboBox::from_label("Renderer Backend")
                .selected_text(&state.renderer)
                .show_ui(ui, |ui| {
                    for backend in ["vulkan", "opengl", "dx12", "metal"] {
                        ui.selectable_value(&mut state.renderer, backend.to_string(), backend);
                    }
                });
            ui.checkbox(&mut state.vsync, "VSync");
            ui.add(egui::Slider::new(&mut state.fps_cap, 30..=240).text("FPS Cap"));
            ui.add_space(20.0);
            if ui.button("Back").clicked() {
                next_state.set(GameScene::MainMenu);
            }
        });
    });
}

fn options_cleanup(mut commands: Commands) {
    commands.remove_resource::<OptionsState>();
}
