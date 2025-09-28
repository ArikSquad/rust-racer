use avian3d::prelude::*;
use bevy::prelude::*;
// use bevy::render::primitives::Aabb; // not needed directly
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_egui::EguiPlugin;
use bevy_framepace::FramepacePlugin;
use bevy_skein::SkeinPlugin;
use bevy::render::mesh::MeshAabb;
pub mod fps_controller;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Interactable;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PhysicsObject;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Ground;

mod scenes;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.5, 0.75, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Unnamed Project".into(),
                resolution: (1280., 720.).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            PhysicsPlugins::default(),
            EguiPlugin::default(),
            FramepacePlugin,
            AtmospherePlugin,
            SkeinPlugin::default(),
            fps_controller::FpsControllerPlugin,
            scenes::ScenePlugin,
        ))
        .insert_resource(fps_controller::LookState::default())
        .register_type::<fps_controller::Player>()
        .register_type::<fps_controller::Npc>()
        .register_type::<Interactable>()
        .register_type::<PhysicsObject>()
        .register_type::<Ground>()
        .insert_state(scenes::GameScene::Splash)
        .add_systems(Startup, (setup_lighting, setup_ui_camera))
        .add_systems(Update, apply_physics_object_system)
        .add_systems(Update, apply_ground_collider_system)
        .run();
}

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}

// asked ai to fix this but it wasn't able to...
fn apply_ground_collider_system(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut q: Query<
        (
            Entity,
            Option<&Mesh3d>,
            Option<&Collider>,
            Option<&RigidBody>,
            Option<&GlobalTransform>,
            &mut Transform,
        ),
        With<Ground>,
    >,
) {
    for (e, mesh_handle_opt, collider_opt, rb_opt, gt_opt, mut t) in q.iter_mut() {
        if !matches!(rb_opt, Some(RigidBody::Static)) {
            commands.entity(e).insert(RigidBody::Static);
        }
        if collider_opt.is_none() {
            let (scale, _rot, trans) = if let Some(gt) = gt_opt { gt.to_scale_rotation_translation() } else { (Vec3::ONE, Quat::IDENTITY, Vec3::ZERO) };
            let mut he = Vec3::splat(0.5);
            let mut center_offset = Vec3::ZERO;
            if let Some(mesh_h) = mesh_handle_opt {
                if let Some(mesh) = meshes.get(mesh_h) {
                    if let Some(aabb) = mesh.compute_aabb() {
                        he = aabb.half_extents.into();
                        center_offset = aabb.center.into();
                    }
                }
            }
            if center_offset != Vec3::ZERO {
                let offset = center_offset * scale;
                t.translation += offset;
            }
            let mut half_extents = he * scale;
            half_extents.y = half_extents.y.max(0.25);
            info!(
                "Ground collider added: entity={:?}, pos=({:.2},{:.2},{:.2}), mesh_he=({:.2},{:.2},{:.2}), center_offset=({:.2},{:.2},{:.2}), scale=({:.2},{:.2},{:.2}), half_extents=({:.2},{:.2},{:.2})",
                e, trans.x, trans.y, trans.z, he.x, he.y, he.z, center_offset.x, center_offset.y, center_offset.z, scale.x, scale.y, scale.z, half_extents.x, half_extents.y, half_extents.z
            );
            commands.entity(e).insert(Collider::cuboid(half_extents.x, half_extents.y, half_extents.z));
        }
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        affects_lightmapped_meshes: false,
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 50.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));
}

#[derive(Component)]
struct NeedsCollider;

fn apply_physics_object_system(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut q: Query<
        (
            Entity,
            Option<&Mesh3d>,
            Option<&Collider>,
            Option<&RigidBody>,
            Option<&GlobalTransform>,
            &mut Transform,
        ),
        With<PhysicsObject>,
    >,
) {
    for (e, mesh_handle_opt, collider_opt, rb_opt, gt_opt, mut t) in q.iter_mut() {
        if !matches!(rb_opt, Some(RigidBody::Dynamic)) {
            commands.entity(e).insert(RigidBody::Dynamic);
        }
        if collider_opt.is_none() {
            let (scale, _rot, trans) = if let Some(gt) = gt_opt { gt.to_scale_rotation_translation() } else { (Vec3::ONE, Quat::IDENTITY, Vec3::ZERO) };
            let mut he = Vec3::splat(0.5);
            let mut center_offset = Vec3::ZERO;
            if let Some(mesh_h) = mesh_handle_opt {
                if let Some(mesh) = meshes.get(mesh_h) {
                    if let Some(aabb) = mesh.compute_aabb() {
                        he = aabb.half_extents.into();
                        center_offset = aabb.center.into();
                    }
                }
            }
            if center_offset != Vec3::ZERO {
                let offset = center_offset * scale;
                t.translation += offset;
            }
            let mut half_extents = he * scale;
            half_extents = half_extents.max(Vec3::splat(0.05));
            info!(
                "PhysicsObject collider added: entity={:?}, pos=({:.2},{:.2},{:.2}), mesh_he=({:.2},{:.2},{:.2}), center_offset=({:.2},{:.2},{:.2}), scale=({:.2},{:.2},{:.2}), half_extents=({:.2},{:.2},{:.2})",
                e, trans.x, trans.y, trans.z, he.x, he.y, he.z, center_offset.x, center_offset.y, center_offset.z, scale.x, scale.y, scale.z, half_extents.x, half_extents.y, half_extents.z
            );
            commands.entity(e).insert(Collider::cuboid(half_extents.x, half_extents.y, half_extents.z));
        }
    }
}
