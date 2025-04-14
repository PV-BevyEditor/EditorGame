use bevy::{
    ecs::system::SystemState, prelude::*
};
use js_sys::{JsString, Object, Reflect};
use transform_gizmo_bevy::GizmoTarget;

use crate::{lib::{components::RotationCamera, editorvisibility::EditorVisible}, wasm::definitions::triggerInterfaceCallbacks};

fn trigger(world: &mut World) {
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

pub fn worldFrame(
    world: &mut World,
) {
    let mut mouseButtonInputState: SystemState<Res<ButtonInput<MouseButton>>> = SystemState::new(world);
    let mouseButtonInput = mouseButtonInputState.get(world);

    // Need to debug here, as it seems this might be triggering more than once per mouse press
    if mouseButtonInput.just_pressed(MouseButton::Left) {
        trigger(world);
    }
}

pub fn worldUpdates(
    world: &mut World,
) {
    let mut queryState: SystemState<Query<(
        // Option<Ref<Camera3d>>, // implement this later, as mentioned in editorvisibility.rs, this is not crucial for now
        Option<Ref<DirectionalLight>>,
        Option<Ref<GlobalTransform>>,
        Option<Ref<Mesh3d>>,
        Option<Ref<MeshMaterial3d<StandardMaterial>>>,
        Option<Ref<RayCastPickable>>,
        Option<Ref<RotationCamera>>,
        Option<Ref<Sprite>>,
        Option<Ref<Transform>>,
        Option<Ref<Visibility>>,
    ), With<GizmoTarget>>> = SystemState::new(world);
    let query = queryState.get(world);
    let single = match query.get_single() {
        Ok(query) => query,
        Err(_) => (None, None, None, None, None, None, None, None, None),
    };

    let (
        directionalLight,
        globalTransform,
        mesh3d,
        material,
        rayCastPickable,
        rotationCamera,
        sprite,
        transform,
        visibility,
    ) = single;

    if 
        directionalLight.map_or(false, |r| r.is_changed()) 
        || globalTransform.map_or(false, |r| r.is_changed())
        || mesh3d.map_or(false, |r| r.is_changed())
        || material.map_or(false, |r| r.is_changed())
        || rayCastPickable.map_or(false, |r| r.is_changed())
        || rotationCamera.map_or(false, |r| r.is_changed())
        || sprite.map_or(false, |r| r.is_changed())
        || transform.map_or(false, |r| r.is_changed())
        || visibility.map_or(false, |r| r.is_changed())
    {
        trigger(world);
        return;
    }
}