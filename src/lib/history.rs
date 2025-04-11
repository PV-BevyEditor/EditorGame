use bevy::prelude::*;
use transform_gizmo_bevy::{GizmoResult, GizmoTransform};

use crate::wasm::definitions::consoleLog;

#[derive(Debug, Clone)]
pub enum HistoryItem {
    Transform(GizmoTransform, Transform),
}

pub enum HistoryAction {
    Undo,
    Redo,
    None
}

pub struct History {
    pub past: Vec<HistoryItem>,
    pub future: Vec<HistoryItem>,
    pub action: HistoryAction,
}
impl History {
    pub fn new() -> Self {
        Self {
            past: Vec::new(), // highest index is most recent
            future: Vec::new(), // highest index is first to happen if redo is called
            action: HistoryAction::None, // the action that should happen next frame
        }
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.past.pop() {
            consoleLog("Undoing...");

            match action {
                HistoryItem::Transform(gizmoTransform, mut transform) => {
                    match gizmoTransform.1 {
                        GizmoResult::Translation { delta: _, total } => {
                            consoleLog(&format!("Undoing translation from:\n{:?}", transform.translation));
                            transform.translation -= Vec3::new(total.x as f32, total.y as f32, total.z as f32);
                            consoleLog(&format!("To:\n{:?}", transform.translation));
                        },
                        GizmoResult::Rotation { axis, delta: _, total, is_view_axis: _ } => {
                            transform.rotation *= Quat::from_axis_angle(Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32), -total as f32);
                        },
                        GizmoResult::Scale { total } => {
                            transform.scale /= Vec3::new(total.x as f32, total.y as f32, total.z as f32);
                        },
                        _ => {},
                    };
                },
            };

            self.future.push(action);
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.future.pop() {
            match action {
                HistoryItem::Transform(gizmoTransform, mut transform) => {
                    match gizmoTransform.1 {
                        GizmoResult::Translation { delta: _, total } => {
                            transform.translation += Vec3::new(total.x as f32, total.y as f32, total.z as f32);
                        },
                        GizmoResult::Rotation { axis, delta: _, total, is_view_axis: _ } => {
                            transform.rotation *= Quat::from_axis_angle(Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32), total as f32);
                        },
                        GizmoResult::Scale { total } => {
                            transform.scale *= Vec3::new(total.x as f32, total.y as f32, total.z as f32);
                        },
                        _ => {},
                    };
                },
            };

            self.past.push(action);
        }
    }
}