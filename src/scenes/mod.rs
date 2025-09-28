pub mod splash;
pub mod main_menu;
pub mod options;
pub mod prologue;

use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameScene {
    #[default]
    Splash,
    MainMenu,
    Options,
    Prologue
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
    app.init_state::<GameScene>()
            .add_plugins(splash::SplashPlugin)
            .add_plugins(main_menu::MainMenuPlugin)
            .add_plugins(options::OptionsPlugin)
            .add_plugins(prologue::ProloguePlugin);
    }
}
