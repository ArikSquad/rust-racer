use bevy::prelude::*;
use crate::scenes::GameScene;
use bevy_egui::{EguiContexts, egui, EguiPrimaryContextPass};

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::Splash), splash_setup)
            .add_systems(EguiPrimaryContextPass, splash_system.run_if(in_state(GameScene::Splash)))
            .add_systems(OnExit(GameScene::Splash), splash_cleanup);
    }
}

#[derive(Resource)]
pub struct SplashState {
    pub timer: Timer,
    pub phase: usize,
    pub alpha: f32,
    pub fade_in: bool,
}

fn splash_setup(mut commands: Commands) {
    commands.insert_resource(SplashState {
        timer: Timer::from_seconds(1.5, TimerMode::Repeating),
        phase: 0,
        alpha: 0.0,
        fade_in: true,
    });
}

fn splash_system(
    mut state: ResMut<SplashState>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameScene>>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let dt = time.delta_secs();
    let fade_speed = 1.2;
    let texts = ["MikArt Europe", "Unnamed Project"];
    let max_phase = texts.len();
    let phase = state.phase;
    if phase >= max_phase {
        next_state.set(GameScene::MainMenu);
        return;
    }
    // skip to main menu if space is pressed
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameScene::MainMenu);
        return;
    }
    
    if state.fade_in {
        state.alpha += dt * fade_speed;
        if state.alpha >= 1.0 {
            state.alpha = 1.0;
            state.timer.tick(time.delta());
            if state.timer.is_finished() {
                state.fade_in = false;
                state.timer.reset();
            }
        }
    } else {
        state.alpha -= dt * fade_speed;
        if state.alpha <= 0.0 {
            state.alpha = 0.0;
            state.phase += 1;
            state.fade_in = true;
            state.timer.reset();
        }
    }
    let show_text = if phase < texts.len() { texts[phase] } else { "" };
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return, // no egui context available this frame
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(200.0);
            ui.label(egui::RichText::new(show_text).size(48.0).color(egui::Color32::from_white_alpha((state.alpha * 255.0) as u8)));
        });
    });
}

fn splash_cleanup(mut commands: Commands) {
    commands.remove_resource::<SplashState>();
}
