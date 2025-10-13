use crate::{controller_avian::RenderPlayer, scenes::GameScene};
use crate::components;
use bevy::prelude::*;

pub struct ProloguePlugin;

impl Plugin for ProloguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::Prologue), prologue_setup)
            .add_systems(
                Update,
                prologue_update.run_if(in_state(GameScene::Prologue)),
            )
            .add_systems(
                Update,
                attach_camera_to_player.run_if(in_state(GameScene::Prologue)),
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

fn prologue_update() {
}

fn attach_camera_to_player(
    mut commands: Commands,
    player_q: Query<Entity, With<components::Player>>,
    camera_q: Query<Entity, (With<Camera3d>, Without<RenderPlayer>)>,
) {
    if let Ok(player_ent) = player_q.single() {
        if let Ok(camera_ent) = camera_q.single() {
            commands.entity(camera_ent).insert(RenderPlayer { logical_entity: player_ent });
            info!("Attached camera {:?} to player {:?}", camera_ent, player_ent);
        }
    }
}
