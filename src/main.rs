#![allow(non_snake_case, dead_code, non_upper_case_globals)]

mod lib {
    pub mod assetloader;
    pub mod components;
    pub mod editorconfig;
    pub mod editorvisibility;
    pub mod jscasting;
}
mod wasm {
    pub mod definitions;

    #[cfg(target_arch = "wasm32")]
    pub mod data;
}
mod systems {
    pub mod startup;
    pub mod update;
}

use lib::{
    components::*,
    editorconfig::EditorConfiguration,
};
use wasm::definitions::*;

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // Fifo: only present mode that wasm accepts, so can't actually turn vsync off :pensive:
                    // Immediate: presents frames as soon as possible (uncapped framerate, no vsync)
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }).set(AssetPlugin {
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
        
        .add_systems(Startup, (setup, setupDynamicAssets).chain())
        .add_systems(Update, (mouseInteractions, keyboardInteractions, update).chain())
        
        .run();
}
