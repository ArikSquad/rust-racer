use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::dialog::{DialogState, Npc};
use crate::Interactable;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Door;

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
        Err(_) => return,
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
    let screen_center = ctx.screen_rect().center();
    let radius = if hit.is_some() { 4.0 } else { 2.0 };
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("crosshair"),
    ));
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

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Door>()
            .add_systems(EguiPrimaryContextPass, crosshair_and_interact_system);
    }
}
