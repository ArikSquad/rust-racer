use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Resource)]
pub struct SpawnPoint(pub Vec3);
