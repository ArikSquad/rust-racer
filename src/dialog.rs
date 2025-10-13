use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::components::Player;

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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Npc {
    pub dialogue_id: String,
}

fn find_node<'a>(graph: &'a DialogGraph, id: &str) -> Option<&'a DialogNode> {
    graph.0.iter().find(|n| n.id == id)
}

fn interaction_prompt_system(
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&GlobalTransform, With<Player>>,
    npc_q: Query<(Entity, &GlobalTransform, Option<&DialogStart>), With<Npc>>,
    mut state: ResMut<DialogState>,
) {
    if state.open {
        return;
    }
    if !(keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Enter)) {
        return;
    }
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

fn dialog_typing_system(time: Res<Time>, mut state: ResMut<DialogState>, graph: Res<DialogGraph>) {
    if !state.open {
        return;
    }
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
    if !state.open {
        return;
    }
    let ctx_res = contexts.ctx_mut();
    let ctx = match ctx_res {
        Ok(c) => c,
        Err(_) => return,
    };
    let Some(id) = state.current else { return; };
    let Some(node) = find_node(&graph, id) else { return; };
    let text = node.text.chars().take(state.visible_chars).collect::<String>();
    egui::TopBottomPanel::bottom("dialog_panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_height_range(egui::Rangef::new(180.0, 300.0));
            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
            ui.label(egui::RichText::new(text).size(20.0));
            ui.add_space(6.0);
            for (_i, opt) in node.options.iter().take(4).enumerate() {
                let label = egui::RichText::new(format!("{}", opt.label))
                    .color(egui::Color32::YELLOW)
                    .size(18.0);
                if ui.button(label).clicked() {
                    if let Some(action) = opt.action.clone() {
                        match action {
                            WorldAction::OpenGate => {
                                info!("Gate opened!");
                            }
                            WorldAction::Teleport(pos) => {
                                if let Ok(mut t) = player_q.single_mut() {
                                    t.translation = pos;
                                }
                            }
                            WorldAction::GiveItem(name) => {
                                info!("Received item: {}", name);
                            }
                        }
                    }
                    if let Some(next) = opt.next {
                        state.current = Some(next);
                        state.visible_chars = 0;
                    } else {
                        state.open = false;
                    }
                }
            }
        });
}

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogState>()
            .init_resource::<DialogGraph>()
            .add_systems(Update, interaction_prompt_system)
            .add_systems(Update, dialog_typing_system)
            .add_systems(EguiPrimaryContextPass, dialog_egui_system)
            .add_systems(Startup, setup_default_dialog_graph);
    }
}

fn setup_default_dialog_graph(mut graph: ResMut<DialogGraph>) {
    if graph.0.is_empty() {
        graph.0 = vec![
            DialogNode {
                id: "root",
                text: "Hello there, traveler. What brings you here?".to_string(),
                options: vec![
                    DialogOption { label: "Looking for work".into(), next: Some("job"), action: None },
                    DialogOption { label: "Just passing by".into(), next: Some("bye"), action: None },
                ],
            },
            DialogNode {
                id: "job",
                text: "There's a gate to the east. I can open it for you.".into(),
                options: vec![
                    DialogOption { label: "Open the gate".into(), next: Some("opened"), action: Some(WorldAction::OpenGate) },
                    DialogOption { label: "Teleport me".into(), next: Some("tele"), action: Some(WorldAction::Teleport(Vec3::new(0.0, 2.0, -6.0))) },
                ],
            },
            DialogNode { id: "opened", text: "Done. Anything else?".into(), options: vec![ DialogOption { label: "Thanks".into(), next: Some("bye"), action: None } ] },
            DialogNode { id: "tele", text: "Hold tight...".into(), options: vec![ DialogOption { label: "Whoa!".into(), next: Some("bye"), action: None } ] },
            DialogNode { id: "bye", text: "Safe travels.".into(), options: vec![] },
            DialogNode {
                id: "demo",
                text: "This is a demo dialog on the red cube.".into(),
                options: vec![
                    DialogOption { label: "Nice".into(), next: Some("bye"), action: None },
                    DialogOption { label: "Teleport me".into(), next: Some("bye"), action: Some(WorldAction::Teleport(Vec3::new(1.0, 2.0, 1.0))) },
                ],
            },
        ];
    }
}
