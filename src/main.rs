#![allow(non_snake_case, dead_code)]

mod lib {
    pub mod assetloader;
    pub mod components;
    pub mod editorconfig;
}

use std::collections::HashMap;
use bevy::{
    color::Color, dev_tools::fps_overlay::{
        FpsOverlayConfig, 
        FpsOverlayPlugin,
    }, input::mouse::MouseMotion, prelude::*, render::view::RenderLayers, window::{
        CursorGrabMode, 
        PresentMode, 
        PrimaryWindow, 
        WindowPlugin,
    },
    picking::pointer::PointerInteraction,
};
use bevy_mod_outline::{
    OutlinePlugin,
    OutlineStencil,
    OutlineVolume,
};
use lib::{
    assetloader::*,
    components::*,
    editorconfig::EditorConfiguration,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate, // vsync off
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
        )).insert_resource(MeshPickingSettings {
            require_markers: true,
            ..default()
        })
        
        .add_systems(Startup, (setup, setupDynamicAssets).chain())
        .add_systems(Update, (mouseInteractions, keyboardInteractions, update).chain())
        
        .run();
}

fn setup(
    mut configStore: ResMut<GizmoConfigStore>,
    mut commands: Commands,
) {
    configStore.config_mut::<DefaultGizmoConfigGroup>().0.render_layers = RenderLayers::layer(0);
    commands.spawn(editorconfig::EditorConfiguration::default());

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3., 5., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        RotationCamera,
        RayCastPickable,
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_x(-1.),
            ..default()
        },
    ));
}

fn setupDynamicAssets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    // images: ResMut<Assets<Image>>,
    configQuery: Query<&EditorConfiguration>,
) {
    // commands.spawn((Camera2d::default(), RenderLayers::layer(0)));
    // commands.spawn((
    //     Sprite {
    //         image: loadUserImage(images).unwrap(),
    //         ..default()
    //     },
    //     Transform::default(),
    //     GlobalTransform::default(),
    //     Visibility::default(),
    // ));

    let material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1., 0.5, 0.5),
        ..default()
    });
        
    for mesh in loadUserModel(meshes).unwrap() {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            OutlineStencil {
                enabled: true,
                offset: 0.,
            },
            OutlineVolume {
                colour: configQuery.single().selection.selectionColour,
                width: 3.,
                visible: true,
            },
            RayCastPickable,
        ));
    }
}

fn mouseInteractions(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<RotationCamera>>,
    mut mouseMotionEvents: EventReader<MouseMotion>,
    mut gizmos: Gizmos,
    mouseButtonInput: Res<ButtonInput<MouseButton>>,
    pointers: Query<&PointerInteraction>,
    configQuery: Query<&EditorConfiguration>,
) {
    let mut window = windows.single_mut();
    let config = configQuery.single();

    // Handling consistently pressed buttons
    // Right mouse button:
    if mouseButtonInput.pressed(MouseButton::Right) {
        let mut deltaRotation = Vec2::ZERO;

        for event in mouseMotionEvents.read() {
            deltaRotation +=event.delta
        }

        if deltaRotation != Vec2::ZERO {
            let mut transform = query.single_mut();
            let yaw = Quat::from_rotation_y(-deltaRotation.x * 0.01);
            let pitch = Quat::from_rotation_x(-deltaRotation.y * 0.01);
            transform.rotation = yaw * transform.rotation * pitch;
        }
    }


    // Handling buttons pressed during last frame
    if mouseButtonInput.just_pressed(MouseButton::Right) {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    // Handling buttons released during last frame
    if mouseButtonInput.just_released(MouseButton::Right) {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }

    // Handling mouse pointer(s)
    for (point, normal) in pointers.iter().filter_map(|interaction| interaction.get_nearest_hit()).filter_map(|(_entity, hit)| hit.position.zip(hit.normal)) {
        gizmos.sphere(point, 0.05, config.selection.selectionColour);
        gizmos.arrow(point, point + normal.normalize() * 0.5, config.selection.highlightColour);
    }
}

fn keyboardInteractions(
    mut cameraTransformQuery: Query<&mut Transform, With<RotationCamera>>,
    keyboardInput: Res<ButtonInput<KeyCode>>,
    configQuery: Query<&EditorConfiguration>,
    time: Res<Time>,
) {
    let cameraSpeed = configQuery.single().camera.cameraSpeed;
    let mut cameraTransform = cameraTransformQuery.single_mut();

    let directionKeyMap: HashMap<KeyCode, Vec3> = [
        (KeyCode::KeyW, -Vec3::Z),
        (KeyCode::ArrowUp, -Vec3::Z),
        (KeyCode::KeyS, Vec3::Z),
        (KeyCode::ArrowDown, Vec3::Z),
        (KeyCode::KeyA, -Vec3::X),
        (KeyCode::ArrowLeft, -Vec3::X),
        (KeyCode::KeyD, Vec3::X),
        (KeyCode::ArrowRight, Vec3::X),
        (KeyCode::KeyQ, -Vec3::Y),
        (KeyCode::KeyE, Vec3::Y),
    ].into_iter().collect();

    for (key, vec) in directionKeyMap.iter() {
        if !keyboardInput.pressed(*key) { continue }

        let forward = cameraTransform.rotation * *vec;
        cameraTransform.translation += forward * cameraSpeed * time.delta_secs();
    }
}

fn update() {}
