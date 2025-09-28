use crate::scenes::GameScene;
use bevy::prelude::*;

pub struct ProloguePlugin;

impl Plugin for ProloguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::Prologue), prologue_setup)
            .add_systems(
                Update,
                prologue_update.run_if(in_state(GameScene::Prologue)),
            );
    }
}
fn prologue_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("levels/World.gltf")),
    ));
}

fn prologue_update() {}
