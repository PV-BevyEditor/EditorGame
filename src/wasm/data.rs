use std::{any, sync::{
    atomic::{
        AtomicU8, 
        Ordering
    }, Arc, RwLock
}};
use bevy::{
    dev_tools::fps_overlay::{
        FpsOverlayConfig,
        FpsOverlayPlugin,
    }, prelude::*, window::PresentMode
};
use bevy_mod_outline::OutlinePlugin;
use transform_gizmo_bevy::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    consoleLog,
    lib::history::*,
    systems::{
        startup::*,
        update::*,
    },
};


// GizmoConfiguration
pub const orientationIsGlobalBit: u8 = 0b0000_0001;
pub const scaleIsVisibleBit: u8 = 0b0000_0010;
pub const translationIsVisibleBit: u8 = 0b0000_0100;
pub const rotationIsVisibleBit: u8 = 0b0000_1000;

pub const gizmoModeMappings: [(u8, &[GizmoMode]); 3] = [
    (scaleIsVisibleBit, &[
        GizmoMode::ScaleUniform,
        GizmoMode::ScaleX,
        GizmoMode::ScaleY,
        GizmoMode::ScaleZ,
    ]),
    (translationIsVisibleBit, &[
        GizmoMode::TranslateView,
        GizmoMode::TranslateX,
        GizmoMode::TranslateY,
        GizmoMode::TranslateZ,
    ]),
    (rotationIsVisibleBit, &[
        GizmoMode::RotateView,
        GizmoMode::RotateX,
        GizmoMode::RotateY,
        GizmoMode::RotateZ,
    ]),
];

#[derive(Resource)]
pub struct CustomGizmoOptions {
    pub gizmoFlags: Arc<AtomicU8>,
}

#[derive(Resource)]
pub struct PreviousCustomGizmoOptions {
    pub gizmoFlags: Arc<AtomicU8>,
}

// #[derive(Resource)]
pub struct BinaryDataQueue {
    pub model: RwLock<Option<Vec<u8>>>,
    pub image: RwLock<Option<Vec<u8>>>,
}

// #[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Runner {
    gizmoFlags: Arc<AtomicU8>,
    binaryData: Arc<BinaryDataQueue>,
    history: Arc<RwLock<History>>,
}

#[derive(Resource)]
pub struct RunnerWrapper {
    // pub runner: Arc<Runner>,
    pub binaryData: Arc<BinaryDataQueue>,
    pub history: Arc<RwLock<History>>,
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct PropertyUpdateInfo {
    pub componentName: &str,
    pub property: &str,
    pub value: any,
}

// #[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Runner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Runner {
        consoleLog("Creating a runner");
                
        Runner {
            gizmoFlags: Arc::new(AtomicU8::new(translationIsVisibleBit)),
            binaryData: Arc::new(BinaryDataQueue {
                model: RwLock::new(None),
                image: RwLock::new(None),
            }),
            history: Arc::new(RwLock::new(History::new())),
        }
    }

    #[wasm_bindgen]
    pub fn startGame(&self) {
        App::new()
            .add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        // Fifo: only present mode that wasm accepts, so can't actually turn vsync off :pensive:
                        // Immediate: presents frames as soon as possible (uncapped framerate, no vsync)
                        present_mode: PresentMode::Fifo,
                        
                        // when inserting the game into a full webpage with a canvas, release mode should be active, debug mode means we are in `cargo run`, so there won't be any canvas element prepared for the game upfront
                        #[cfg(not(debug_assertions))]
                        canvas: Some("#game_canvas".into()),
                        
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
                FpsOverlayPlugin {
                    config: FpsOverlayConfig {
                        text_config: TextFont {
                            font_size: 20.,
                            ..default()
                        },
                        enabled: false,
                        ..default()
                    }
                },
                OutlinePlugin,
                MeshPickingPlugin,
                TransformGizmoPlugin,
            ))

            .insert_resource(MeshPickingSettings {
                require_markers: true,
                ..default()
            })
            .insert_resource(GizmoOptions {
                gizmo_orientation: GizmoOrientation::Local,
                ..default()
            })
            .insert_resource(CustomGizmoOptions {
                gizmoFlags: self.gizmoFlags.clone(),
            })
            .insert_resource(PreviousCustomGizmoOptions {
                gizmoFlags: Arc::new(AtomicU8::new(self.gizmoFlags.load(Ordering::SeqCst))),
            })
            .insert_resource(RunnerWrapper {
                // Since we're just cloning the arcs, we're creating new references, without actually duplicating any potential data
                binaryData: self.binaryData.clone(),
                history: self.history.clone()
            })

            .add_systems(Startup, (setup, setupDynamicAssets).chain())
            .add_systems(Update, (syncData, mouseInteractions, keyboardInteractions, handleHistory, handleUndoRedo).chain())
            .add_systems(PostUpdate, worldFrame)
            
            .run();
    }

    #[wasm_bindgen]
    pub fn setGizmoOptions(&self, flags: u8) {
        self.gizmoFlags.store(flags, Ordering::SeqCst);
    }

    #[wasm_bindgen]
    pub fn loadModel(&self, bytes: &[u8]) {
        if let Ok(mut data) = self.binaryData.model.write() {
            *data = Some(bytes.to_vec());
        }
    }

    #[wasm_bindgen]
    pub fn sendEvent(&self, eventType: &str, info: &str) {
        match eventType {
            "undo" => {
                if let Ok(mut history) = self.history.write() {
                    history.action = HistoryAction::Undo;
                } else {
                    consoleLog("Tried writing simultaneously while undoing");
                }
            },
            "redo" => {
                if let Ok(mut history) = self.history.write() {
                    history.action = HistoryAction::Redo;
                    consoleLog("Tried writing simultaneously while redoing");
                }
            },
            "read" => {
                if let Ok(history) = self.history.read() {
                    consoleLog(&format!("Past: {:?}\n\nFuture: {:?}", history.past, history.future));
                }
            },
            "setProperty" => {
                let info: PropertyUpdateInfo = serde_json::from_str(info).unwrap();
            },
            _ => {}
        };
    }
}
