use bevy::{
    ecs::system::SystemState, input::mouse::MouseMotion, picking::pointer::PointerInteraction, prelude::*, window::{
        CursorGrabMode, 
        PrimaryWindow, 
    }
};
use bevy_mod_outline::{
    OutlineStencil,
    OutlineVolume,
};
use js_sys::{Object, Reflect, JsString};
use std::collections::HashMap;
use transform_gizmo_bevy::{prelude::*, GizmoTransform};

#[allow(unused_imports)]
use crate::{
    lib::editorvisibility::EditorVisible, wasm::definitions::{addToHistory, consoleLog}, EditorConfiguration, RotationCamera
};
#[cfg(target_arch = "wasm32")]
use {
    crate::{
        wasm::data::*,
        lib::{
            assetloader::*,
            history::*,
        },
        triggerInterfaceCallbacks,
        // consoleLog,
    },
    std::sync::atomic::Ordering,
};

pub fn worldFrame(
    world: &mut World,
) {
    let mut mouseButtonInputState: SystemState<Res<ButtonInput<MouseButton>>> = SystemState::new(world);
    let mouseButtonInput = mouseButtonInputState.get(world);

    // Need to debug here, as it seems this might be triggering more than once per mouse press
    if mouseButtonInput.just_pressed(MouseButton::Left) {
        let mut gizmoTargetState: SystemState<Query<Entity, With<GizmoTarget>>> = SystemState::new(world);
        let gizmoTarget = match gizmoTargetState.get(world).get_single() {
            Ok(target) => target,
            Err(_) => return triggerInterfaceCallbacks("properties", vec![]),
        };

        let mut infoVec: Vec<Object> = vec![];
        for (_, component) in world.inspect_entity(gizmoTarget).enumerate() {
            if !component.isEditorVisible() { continue; }

            let obj = Object::new();

            Reflect::set(&obj, &JsString::from("name"), &JsString::from(component.name())).unwrap();
            Reflect::set(&obj, &JsString::from("info"), &component.getInfo(world, gizmoTarget).into()).unwrap();

            infoVec.push(obj);
        }
        
        triggerInterfaceCallbacks("properties", infoVec);
    }
}

pub fn mouseInteractions(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<RotationCamera>>,
    mut mouseMotionEvents: EventReader<MouseMotion>,
    mut clickables: Query<(Entity, &mut OutlineVolume), (With<Mesh3d>, With<OutlineStencil>)>,
    mut commands: Commands,
    gizmoTargets: Query<Entity, With<GizmoTarget>>,
    mouseButtonInput: Res<ButtonInput<MouseButton>>,
    pointers: Query<&PointerInteraction>,
) {
    let mut window = windows.single_mut();

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
    if mouseButtonInput.just_pressed(MouseButton::Left) {
        // Remove GizmoTargets in existing places and make selection outlines invisible
        for entity in gizmoTargets.iter() {
            commands.entity(entity).remove::<GizmoTarget>();
        }
        for mut clickable in clickables.iter_mut() {
            clickable.1.visible = false;
        }

        if let Some((point, _normal)) = pointers.iter().filter_map(|interaction| interaction.get_nearest_hit()).into_iter().nth(0) {
            if let Ok((entity, mut outlineVolume)) = clickables.get_mut(*point) {
                // Handle outline and gizmos
                outlineVolume.visible = true;
                commands.entity(entity).insert(GizmoTarget::default());
            }
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
}

pub fn keyboardInteractions(
    mut cameraTransformQuery: Query<&mut Transform, With<RotationCamera>>,
    mut gizmoSettings: ResMut<GizmoOptions>,
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

    if keyboardInput.pressed(KeyCode::ControlLeft) && keyboardInput.just_pressed(KeyCode::KeyL) {
        gizmoSettings.gizmo_orientation = if gizmoSettings.gizmo_orientation == GizmoOrientation::Global { GizmoOrientation::Local } else { GizmoOrientation::Global }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn syncData(
    mut gizmoOptions: ResMut<GizmoOptions>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    images: ResMut<Assets<Image>>,
    last: Res<PreviousCustomGizmoOptions>,
    sync: Res<CustomGizmoOptions>,
    runner: Res<RunnerWrapper>,
    materials: Query<&MeshMaterial3d<StandardMaterial>>,
    configQuery: Query<&EditorConfiguration>,
) {
    // Handling Gizmo option flags
    let prevFlags = last.gizmoFlags.load(Ordering::SeqCst);
    let flags = sync.gizmoFlags.load(Ordering::SeqCst);
    
    if prevFlags != flags {
        last.gizmoFlags.store(flags, Ordering::SeqCst);
        
        gizmoOptions.gizmo_orientation = if flags & orientationIsGlobalBit != 0 { GizmoOrientation::Global } else { GizmoOrientation::Local };
        
        let mut enumSet: EnumSet<GizmoMode> = EnumSet::new();
        for (bit, modes) in gizmoModeMappings {
            if flags & bit != 0 {
                enumSet.extend(modes.iter().cloned());
            }
        }
        
        gizmoOptions.gizmo_modes = enumSet;
    }

    // Handling model loading
    if let Ok(mut modelGuard) = runner.binaryData.model.write() {
        if let Some(model) = modelGuard.take() {
            let importedMeshes = loadModel(meshes, &model).unwrap();

            for meshHandle in importedMeshes.iter() {
                let material = materials.single();

                commands.spawn((
                    Mesh3d(meshHandle.clone()),
                    material.clone(),
                    OutlineStencil {
                        enabled: true,
                        offset: 0.,
                    },
                    OutlineVolume {
                        colour: configQuery.single().selection.selectionColour,
                        width: 3.,
                        visible: false,
                    },
                    RayCastPickable,
                    GizmoTarget::default(),
                ));
            }
        }
    }
    if let Ok(mut imageGuard) = runner.binaryData.image.write() {
        if let Some(image) = imageGuard.take() {
            loadImage(images, &image).unwrap();
        }
    }
}

pub fn update(
    mut gizmoEvents: EventReader<GizmoTransform>,
    mut transformableEntityQuery: Query<(Entity, &mut Transform)>,
    mut runnerWrapper: ResMut<RunnerWrapper>,
) {
    let events = gizmoEvents.read().collect::<Vec<&GizmoTransform>>();
    let runner = runnerWrapper.as_mut();
    
    if !events.is_empty() {
        for gizmoTransform in events {
            for (entity, transform) in transformableEntityQuery.iter_mut() {
                if entity != gizmoTransform.0 { continue; }
    
                if let Ok(mut history) = runner.history.write() {
                    history.future.clear();
                    history.past.push(
                        HistoryItem::Transform(
                            *gizmoTransform,
                            *transform,
                        ),
                    );
                }
            }
        }
    }

    if let Ok(mut history) = runner.history.write() {
        if matches!(history.action, HistoryAction::None) { return; }
        
        let isUndo = matches!(history.action, HistoryAction::Undo);
        history.action = HistoryAction::None;

        if let Some(gizmoTransform) = if isUndo { history.past.pop() } else { history.future.pop() } {
            for (entity, mut transform) in transformableEntityQuery.iter_mut() {
                match gizmoTransform {
                    HistoryItem::Transform(gizmoTransform, _) => {
                        if entity != gizmoTransform.0 { continue; }
    
                        if isUndo {
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
                        } else {
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
                        }
                    },
                };
            }
        }
    }
}
