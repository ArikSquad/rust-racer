use crate::components::SpawnPoint;
use crate::controller_avian::*;
use avian3d::prelude::*;
use bevy::camera::Exposure;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings};
use bevy::post_process::bloom::Bloom;
use bevy::window::CursorGrabMode;
use bevy::{
    camera::primitives::MeshAabb,
    core_pipeline::tonemapping::Tonemapping,
    light::{light_consts::lux, AtmosphereEnvironmentMapLight},
    prelude::*,
};
use bevy_egui::egui;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::bevy_inspector;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_skein::SkeinPlugin;
use std::f32::consts::TAU;
pub mod components;
pub mod controller_avian;
pub mod dialog;
pub mod interact;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Interactable;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PhysicsObject;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Ground;

#[derive(Component)]
pub struct GroundColliderComputed;

mod scenes;

#[derive(Resource, Default)]
pub struct InspectorVisible(pub bool);

fn inspector_ui(world: &mut World) {
    let visible = world
        .get_resource::<InspectorVisible>()
        .map(|r| r.0)
        .unwrap_or(false);
    if !visible {
        return;
    }

    let mut egui_ctx = world
        .query_filtered::<&mut bevy_egui::EguiContext, With<bevy_egui::PrimaryEguiContext>>()
        .single(world)
        .expect("EguiContext not found")
        .clone();

    egui::Window::new("World Inspector")
        .scroll(true)
        .show(egui_ctx.get_mut(), |ui| {
            bevy_inspector::ui_for_world(world, ui);
        });
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.5, 0.75, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Unnamed Project".into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            PhysicsPlugins::default(),
            EguiPlugin::default(),
            DefaultInspectorConfigPlugin,
            SkeinPlugin::default(),
            controller_avian::FpsControllerPlugin,
            scenes::ScenePlugin,
            PhysicsDebugPlugin,
            FpsOverlayPlugin::default(),
        ))
        .insert_resource(SpawnPoint(Vec3::ZERO))
        .insert_resource(InspectorVisible(false))
        .register_type::<components::Player>()
        .register_type::<dialog::Npc>()
        .register_type::<Interactable>()
        .register_type::<PhysicsObject>()
        .register_type::<Ground>()
        .insert_state(scenes::GameScene::Splash)
        .add_systems(Startup, (setup_lighting, setup_avian_player_and_camera))
        .add_systems(Update, apply_physics_object_system)
        .add_systems(Update, apply_ground_collider_system)
        .add_systems(EguiPrimaryContextPass, inspector_ui)
        .add_plugins((dialog::DialogPlugin, interact::InteractPlugin))
        .add_observer(|add: On<Add, components::Player>, mut commands: Commands| {
            let height = 3.0;
            commands.entity(add.entity).insert((
                Collider::cylinder(0.5, height),
                Friction {
                    dynamic_coefficient: 0.0,
                    static_coefficient: 0.0,
                    combine_rule: CoefficientCombine::Min,
                },
                Restitution {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombine::Min,
                },
                LinearVelocity::ZERO,
                RigidBody::Dynamic,
                Sleeping,
                LockedAxes::ROTATION_LOCKED,
                Mass(1.0),
                GravityScale(0.0),
                Transform::from_translation(SpawnPoint(Vec3::ZERO).0),
                LogicalPlayer,
                FpsControllerInput {
                    pitch: -TAU / 12.0,
                    yaw: TAU * 5.0 / 8.0,
                    ..default()
                },
                FpsController {
                    air_acceleration: 80.0,
                    ..default()
                },
                CameraConfig {
                    height_offset: -0.5,
                },
            ));
        })
        .run();
}

fn setup_avian_player_and_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Atmosphere::EARTH,
        AtmosphereSettings {
            rendering_method: AtmosphereMode::Raymarched,
            ..default()
        },
        Exposure::SUNLIGHT,
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
    ));
}

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
        (With<Ground>, Without<GroundColliderComputed>),
    >,
    children_q: Query<&Children>,
    mesh_and_gt_q: Query<(&Mesh3d, &GlobalTransform)>,
) {
    fn find_descendant_world_aabb(
        start: Entity,
        children_q: &Query<&Children>,
        mesh_and_gt_q: &Query<(&Mesh3d, &GlobalTransform)>,
        meshes: &Res<Assets<Mesh>>,
    ) -> Option<(Vec3, Vec3)> {
        let mut todo = vec![start];
        while let Some(curr) = todo.pop() {
            if let Ok(children) = children_q.get(curr) {
                for idx in 0..children.len() {
                    let child = children[idx];
                    if let Ok((mesh3d, child_gt)) = mesh_and_gt_q.get(child) {
                        if let Some(mesh) = meshes.get(&mesh3d.0) {
                            if let Some(aabb) = mesh.compute_aabb() {
                                let center: Vec3 = aabb.center.into();
                                let he: Vec3 = aabb.half_extents.into();
                                let corners: [Vec3; 8] = [
                                    center + Vec3::new(he.x, he.y, he.z),
                                    center + Vec3::new(he.x, he.y, -he.z),
                                    center + Vec3::new(he.x, -he.y, he.z),
                                    center + Vec3::new(he.x, -he.y, -he.z),
                                    center + Vec3::new(-he.x, he.y, he.z),
                                    center + Vec3::new(-he.x, he.y, -he.z),
                                    center + Vec3::new(-he.x, -he.y, he.z),
                                    center + Vec3::new(-he.x, -he.y, -he.z),
                                ];
                                let mut world_min = Vec3::splat(f32::MAX);
                                let mut world_max = Vec3::splat(f32::MIN);
                                for c in corners.iter() {
                                    let wc = child_gt.affine().transform_point3(*c);
                                    world_min = world_min.min(wc);
                                    world_max = world_max.max(wc);
                                }
                                let world_center = (world_min + world_max) * 0.5;
                                let world_half = world_max - world_min;
                                return Some((world_center, world_half));
                            } else {
                                info!("mesh for entity {:?} has mesh asset but compute_aabb returned None", start);
                            }
                        } else {
                            info!(
                                "mesh handle present but Mesh asset not loaded for entity {:?}",
                                start
                            );
                        }
                    }
                    if let Ok(grand_children) = children_q.get(child) {
                        for j in 0..grand_children.len() {
                            let gc = grand_children[j];
                            todo.push(gc);
                        }
                    }
                }
            }
        }
        None
    }

    for (e, mesh_handle_opt, collider_opt, rb_opt, gt_opt, mut t) in q.iter_mut() {
        if let Some(mesh_h) = mesh_handle_opt {
            match meshes.get(mesh_h) {
                Some(mesh) => {
                    if let Some(aabb) = mesh.compute_aabb() {
                        info!(
                            "Ground entity {:?}: mesh asset loaded and compute_aabb returned Some(center=({:.2},{:.2},{:.2}), he=({:.2},{:.2},{:.2}))",
                            e,
                            aabb.center.x,
                            aabb.center.y,
                            aabb.center.z,
                            aabb.half_extents.x,
                            aabb.half_extents.y,
                            aabb.half_extents.z
                        );
                    } else {
                        info!(
                            "Ground entity {:?}: mesh asset loaded but compute_aabb returned None",
                            e
                        );
                    }
                }
                None => {
                    info!(
                        "Ground entity {:?}: mesh handle present but Mesh asset not yet loaded",
                        e
                    );
                }
            }
        }

        let descendant_world_aabb = if mesh_handle_opt.is_none() {
            let found = find_descendant_world_aabb(e, &children_q, &mesh_and_gt_q, &meshes);
            if found.is_none() {
                info!("ground entity {:?}: no descendant with mesh found", e);
            }
            found
        } else {
            None
        };

        if !matches!(rb_opt, Some(RigidBody::Static)) {
            commands.entity(e).insert(RigidBody::Static);
        }

        if collider_opt.is_none() {
            let (scale, _rot, trans) = if let Some(gt) = gt_opt {
                gt.to_scale_rotation_translation()
            } else {
                (t.scale, Quat::IDENTITY, t.translation)
            };

            let mesh_he = Vec3::splat(0.5);
            let center_offset = Vec3::ZERO;
            let half_extents = if let Some((world_center, world_half)) = descendant_world_aabb {
                if let Some(parent_gt) = gt_opt {
                    let local_center = parent_gt.affine().inverse().transform_point3(world_center);
                    t.translation += local_center;
                    let parent_scale = scale.abs();
                    Vec3::new(
                        (world_half.x / parent_scale.x).abs(),
                        (world_half.y / parent_scale.y).abs(),
                        (world_half.z / parent_scale.z).abs(),
                    )
                } else {
                    world_half
                }
            } else {
                Vec3::splat(0.5)
            };

            info!(
                "Ground collider added: entity={:?}, pos=({:.2},{:.2},{:.2}), mesh_he=({:.2},{:.2},{:.2}), center_offset=({:.2},{:.2},{:.2}), scale=({:.2},{:.2},{:.2}), half_extents=({:.2},{:.2},{:.2})",
                e,
                trans.x,
                trans.y,
                trans.z,
                mesh_he.x,
                mesh_he.y,
                mesh_he.z,
                center_offset.x,
                center_offset.y,
                center_offset.z,
                scale.x,
                scale.y,
                scale.z,
                half_extents.x,
                half_extents.y,
                half_extents.z
            );

            commands.entity(e).insert(Collider::cuboid(
                half_extents.x,
                half_extents.y,
                half_extents.z,
            ));
            commands.entity(e).insert(GroundColliderComputed);
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
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 50.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));
}

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
            let (scale, _rot, trans) = if let Some(gt) = gt_opt {
                gt.to_scale_rotation_translation()
            } else {
                (Vec3::ONE, Quat::IDENTITY, Vec3::ZERO)
            };
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
                "PhysicsObject colliders added: entity={:?}, pos=({:.2},{:.2},{:.2}), mesh_he=({:.2},{:.2},{:.2}), center_offset=({:.2},{:.2},{:.2}), scale=({:.2},{:.2},{:.2}), half_extents=({:.2},{:.2},{:.2})",
                e, trans.x, trans.y, trans.z, he.x, he.y, he.z, center_offset.x, center_offset.y, center_offset.z, scale.x, scale.y, scale.z, half_extents.x, half_extents.y, half_extents.z
            );
            commands.entity(e).insert(Collider::cuboid(
                half_extents.x,
                half_extents.y,
                half_extents.z,
            ));
        }
    }
}
