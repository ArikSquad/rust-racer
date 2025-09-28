use bevy_egui::{EguiContexts, egui, EguiPrimaryContextPass};
use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::resource_exists;
use bevy::input::mouse::MouseMotion;
use crate::Interactable;
use crate::scenes::GameScene;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Door;

fn find_node<'a>(graph: &'a DialogGraph, id: &str) -> Option<&'a DialogNode> {
    graph.0.iter().find(|n| n.id == id)
}

fn first_person_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    mut look: ResMut<LookState>,
    mut q: Query<&mut Transform, With<Player>>,
    mut vel_q: Query<&mut avian3d::prelude::LinearVelocity, With<Player>>,
) {
    if let Ok(mut t) = q.single_mut() {
        let mut delta = Vec2::ZERO;
        for m in mouse.read() { delta += m.delta; }
        let sensitivity = 0.0025; // maybe config?
        look.yaw -= delta.x as f32 * sensitivity;
        look.pitch = (look.pitch - delta.y as f32 * sensitivity).clamp(-1.45, 1.45);
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, look.yaw);
        t.rotation = yaw_rot;

        let mut dir = Vec3::ZERO;
        let forward = (yaw_rot * Vec3::NEG_Z).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) { dir += forward; }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) { dir -= forward; }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) { dir -= right; }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir += right; }
        let wish_dir = if dir.length_squared() > 0.0 { dir.normalize() } else { Vec3::ZERO };
        let speed = if keys.pressed(KeyCode::ShiftLeft) { 9.0 } else { 5.0 };
        if let Ok(mut lv) = vel_q.single_mut() {
            let current_y = lv.0.y;
            let mut new_vel = Vec3::new(wish_dir.x * speed, current_y, wish_dir.z * speed);
            if keys.just_pressed(KeyCode::Space) && current_y.abs() < 0.05 {
                new_vel.y = 5.5;
            }
            lv.0 = new_vel;
        }
    }
}

fn camera_follow_system(
    look: Option<Res<LookState>>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut cam_q: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let Some(look) = look else { return };
    if let (Ok(p_g), Ok(mut cam_t)) = (player_q.single(), cam_q.single_mut()) {
        let eye_height = 1.6;
        cam_t.translation = p_g.translation() + Vec3::Y * eye_height;
        let rot = Quat::from_axis_angle(Vec3::Y, look.yaw) * Quat::from_axis_angle(Vec3::X, look.pitch);
        cam_t.rotation = rot;
    }
}

fn ensure_player_physics_system(
    mut commands: Commands,
    q: Query<(Entity, Option<&avian3d::prelude::RigidBody>, Option<&avian3d::prelude::Collider>, Option<&avian3d::prelude::LinearVelocity>), With<Player>>,
) {
    if let Ok((e, rb_opt, col_opt, lv_opt)) = q.single() {
        let mut ecmd = commands.entity(e);
        if rb_opt.is_none() { ecmd.insert(avian3d::prelude::RigidBody::Dynamic); }
        if col_opt.is_none() { ecmd.insert(avian3d::prelude::Collider::cuboid(0.4, 0.9, 0.4)); }
        if lv_opt.is_none() { ecmd.insert(avian3d::prelude::LinearVelocity(Vec3::ZERO)); }
    }
}

fn interaction_prompt_system(
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&GlobalTransform, With<Player>>,
    npc_q: Query<(Entity, &GlobalTransform, Option<&DialogStart>), With<Npc>>,
    mut state: ResMut<DialogState>,
) {
    if state.open { return; }
    if !(keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Enter)) { return; }
    if let Ok(p) = player_q.single() {
        let mut chosen: Option<&'static str> = None;
        for (_e, n_t, start) in npc_q.iter() {
            let dist = p.translation().distance(n_t.translation());
            if dist < 2.5 {
                chosen = Some(start.map(|s| s.0).unwrap_or("root"));
                break;
            }
        }
        if let Some(id) = chosen {
            state.open = true;
            state.current = Some(id);
            state.visible_chars = 0;
        }
    }
}

fn dialog_typing_system(
    time: Res<Time>,
    mut state: ResMut<DialogState>,
    graph: Res<DialogGraph>,
) {
    if !state.open { return; }
    let Some(id) = state.current else { return; };
    let Some(node) = find_node(&graph, id) else { return; };
    let new_count = (state.visible_chars as f32 + state.typing_speed * time.delta_secs()) as usize;
    state.visible_chars = new_count.min(node.text.len());
}

fn dialog_egui_system(
    mut contexts: EguiContexts,
    graph: Res<DialogGraph>,
    mut state: ResMut<DialogState>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    if !state.open { return; }
    let ctx_res = contexts.ctx_mut();
    let ctx = match ctx_res {
        Ok(c) => c,
        Err(_) => return, // no egui context available this frame
    };
    let Some(id) = state.current else { return; };
    let Some(node) = find_node(&graph, id) else { return; };
    let text = node.text.chars().take(state.visible_chars).collect::<String>();
    egui::TopBottomPanel::bottom("dialog_panel").resizable(false).show(ctx, |ui| {
        ui.set_height_range(egui::Rangef::new(180.0, 300.0));
        ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
        ui.label(egui::RichText::new(text).size(20.0));
        ui.add_space(6.0);
        for (_i, opt) in node.options.iter().take(4).enumerate() {
            let label = egui::RichText::new(format!("{}", opt.label)).color(egui::Color32::YELLOW).size(18.0);
            if ui.button(label).clicked() {
                if let Some(action) = opt.action.clone() {
                    match action {
                        WorldAction::OpenGate => { info!("Gate opened!"); }
                        WorldAction::Teleport(pos) => { if let Ok(mut t) = player_q.single_mut() { t.translation = pos; } }
                        WorldAction::GiveItem(name) => { info!("Received item: {}", name); }
                    }
                }
                if let Some(next) = opt.next { state.current = Some(next); state.visible_chars = 0; } else { state.open = false; }
            }
        }
    });
}

fn crosshair_and_interact_system(
    mut contexts: EguiContexts,
    camera_q: Query<(&GlobalTransform, &Camera), With<Camera>>,
    interactable_q: Query<(Entity, &GlobalTransform), With<Interactable>>,
    npc_q: Query<&Npc>,
    door_q: Query<&Door>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut dialog_state: ResMut<DialogState>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return, // no egui context available this frame
    };
    let mut hit = None;
    if let Ok((cam_transform, _cam)) = camera_q.single() {
        let origin = cam_transform.translation();
        let dir = cam_transform.forward();
        let max_dist = 5.0;
        for (entity, t) in interactable_q.iter() {
            let to = t.translation() - origin;
            let proj = to.dot(*dir);
            if proj > 0.0 && proj < max_dist {
                let closest = origin + (*dir) * proj;
                if (closest - t.translation()).length() < 0.5 {
                    hit = Some(entity);
                    break;
                }
            }
        }
    }
    // crosshair
    let screen_center = ctx.screen_rect().center();
    let radius = if hit.is_some() { 4.0 } else { 2.0 };
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("crosshair")));
    painter.circle_filled(screen_center, radius, egui::Color32::WHITE);
    if hit.is_some() {
        let hint_offset = egui::vec2(0.0, 20.0);
        painter.text(
            screen_center + hint_offset,
            egui::Align2::CENTER_TOP,
            "Press E to interact",
            egui::FontId::proportional(16.0),
            egui::Color32::YELLOW,
        );
    }
    if let Some(target) = hit {
        if keys.just_pressed(KeyCode::KeyE) {
            if let Ok(npc) = npc_q.get(target) {
                if !npc.dialogue_id.is_empty() {
                    dialog_state.open = true;
                    dialog_state.current = Some(Box::leak(npc.dialogue_id.clone().into_boxed_str()));
                    dialog_state.visible_chars = 0;
                }
            }

            if mouse.just_pressed(MouseButton::Right) {
                if let Ok(_door) = door_q.get(target) {
                    info!("Door interacted: {:?}", target);
                }
            }

            info!("Interacted with entity: {:?}", target);
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Npc {
    pub dialogue_id: String,
}

#[derive(Component)]
pub struct DialogStart(pub &'static str);

#[derive(Resource, Default)]
pub struct DialogState {
    pub open: bool,
    pub current: Option<&'static str>,
    pub visible_chars: usize,
    pub typing_speed: f32,
}

#[derive(Resource, Default)]
pub struct DialogGraph(pub Vec<DialogNode>);

#[derive(Clone)]
pub struct DialogOption {
    pub label: String,
    pub next: Option<&'static str>,
    pub action: Option<WorldAction>,
}

#[derive(Clone)]
pub struct DialogNode {
    pub id: &'static str,
    pub text: String,
    pub options: Vec<DialogOption>,
}

#[derive(Clone)]
pub enum WorldAction {
    GiveItem(&'static str),
    OpenGate,
    Teleport(Vec3),
}

#[derive(Component)]
pub struct FollowCam;

#[derive(Resource, Default)]
pub struct LookState { pub yaw: f32, pub pitch: f32 }

pub struct FpsControllerPlugin;

impl Plugin for FpsControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DialogState>()
            .init_resource::<DialogGraph>() 
            .init_resource::<LookState>()
            .add_systems(Startup, setup_default_dialog_graph)
            .add_systems(Update, ensure_player_physics_system)
            .add_systems(Update, first_person_input_system)
            .add_systems(Update, interaction_prompt_system)
            .add_systems(Update, dialog_typing_system)
            .add_systems(EguiPrimaryContextPass, dialog_egui_system)
            .add_systems(EguiPrimaryContextPass, crosshair_and_interact_system.run_if(in_state(GameScene::Prologue)));
    app.add_systems(PostUpdate, camera_follow_system.run_if(resource_exists::<LookState>));
    }
}

// random dialog tests
fn setup_default_dialog_graph(mut graph: ResMut<DialogGraph>) {
    if graph.0.is_empty() {
        graph.0 = vec![
            DialogNode { id: "root", text: "Hello there, traveler. What brings you here?".to_string(), options: vec![
                DialogOption { label: "Looking for work".into(), next: Some("job"), action: None },
                DialogOption { label: "Just passing by".into(), next: Some("bye"), action: None },
            ]},
            DialogNode { id: "job", text: "There's a gate to the east. I can open it for you.".into(), options: vec![
                DialogOption { label: "Open the gate".into(), next: Some("opened"), action: Some(WorldAction::OpenGate) },
                DialogOption { label: "Teleport me".into(), next: Some("tele"), action: Some(WorldAction::Teleport(Vec3::new(0.0,2.0,-6.0))) },
            ]},
            DialogNode { id: "opened", text: "Done. Anything else?".into(), options: vec![
                DialogOption { label: "Thanks".into(), next: Some("bye"), action: None },
            ]},
            DialogNode { id: "tele", text: "Hold tight...".into(), options: vec![
                DialogOption { label: "Whoa!".into(), next: Some("bye"), action: None },
            ]},
            DialogNode { id: "bye", text: "Safe travels.".into(), options: vec![] },
            DialogNode { id: "demo", text: "This is a demo dialog on the red cube.".into(), options: vec![
                DialogOption { label: "Nice".into(), next: Some("bye"), action: None },
                DialogOption { label: "Teleport me".into(), next: Some("bye"), action: Some(WorldAction::Teleport(Vec3::new(1.0,2.0,1.0))) },
            ]},
        ];
    }
}
