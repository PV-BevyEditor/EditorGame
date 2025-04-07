use std::any::TypeId;

use bevy::{core_pipeline::core_3d::Camera3d, ecs::{component::ComponentInfo, entity::Entity, world::World}, pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial}, reflect::reflect_trait, render::{mesh::Mesh3d, view::Visibility}, sprite::Sprite, transform::components::{GlobalTransform, Transform}, utils::HashSet};
use bevy_picking::mesh_picking::RayCastPickable;
use js_sys::Object;
use once_cell::sync::Lazy;
// use bevy_mod_outline::{OutlineStencil, OutlineVolume};
// use transform_gizmo_bevy::{GizmoCamera, GizmoTarget};

use crate::{
    RotationCamera,
    lib::jscasting::IntoJs,
};

use super::jscasting::asJsObject;

#[reflect_trait]
pub trait EditorVisible {
    fn isEditorVisible(&self) -> bool;
    fn getInfo(&self, world: &World, entity: Entity) -> Object;
}

static EditorVisibleTypes: Lazy<HashSet<TypeId>> = Lazy::new(|| {
    let mut set = HashSet::new();

    // set.insert(TypeId::of::<Camera2d>());
    set.insert(TypeId::of::<Camera3d>());
    set.insert(TypeId::of::<DirectionalLight>());
    set.insert(TypeId::of::<GlobalTransform>());
    // set.insert(TypeId::of::<GizmoCamera>()); // these two are not default components, rather ones that are only used in the editor, and probably not in games
    // set.insert(TypeId::of::<GizmoTarget>());
    set.insert(TypeId::of::<Mesh3d>());
    set.insert(TypeId::of::<MeshMaterial3d<StandardMaterial>>());
    // set.insert(TypeId::of::<OutlineStencil>());
    // set.insert(TypeId::of::<OutlineVolume>());
    set.insert(TypeId::of::<RayCastPickable>());
    set.insert(TypeId::of::<RotationCamera>());
    set.insert(TypeId::of::<Sprite>());
    set.insert(TypeId::of::<Transform>());
    set.insert(TypeId::of::<Visibility>());

    set
});



impl EditorVisible for ComponentInfo {
    fn isEditorVisible(&self) -> bool {
        self.type_id().map_or(false, |typeId| EditorVisibleTypes.contains(&typeId))
    }

    // Will only pass information that's crucial, info about components which is mostly not used will be omitted for now
    fn getInfo(&self, world: &World, entity: Entity) -> Object {
        let id = self.type_id().unwrap();
        match id {
            // id if id == TypeId::of::<Camera2d>() => {
            //     Has no properties?
            //     return Object::new();
            // }
            id if id == TypeId::of::<Camera3d>() => {
                // Implement later, no crucial properties
                return Object::new();
            }
            id if id == TypeId::of::<DirectionalLight>() => {
                let lightComponent = world.get::<DirectionalLight>(entity).unwrap();

                return asJsObject(vec![
                    ("colour", lightComponent.color.intoJs().into()),
                    ("intensity", lightComponent.illuminance.into()),
                    ("shadows", lightComponent.shadows_enabled.into()),
                    ("shadowDepthBias", lightComponent.shadow_depth_bias.into()),
                    ("shadowNormalBias", lightComponent.shadow_normal_bias.into()),
                    // Can add soft_shadow_size later, it is availlable only with "experimental_pbr_pcss" feature
                ]);
            }
            id if id == TypeId::of::<GlobalTransform>() => {
                // Only has private fields?
                return Object::new();
            }
            id if id == TypeId::of::<Mesh3d>() => {
                let meshComponent = world.get::<Mesh3d>(entity).unwrap();

                return asJsObject(vec![
                    ("mesh", meshComponent.0.intoJs().into()),
                ]);
            }
            id if id == TypeId::of::<MeshMaterial3d<StandardMaterial>>() => {
                let meshMaterialComponent = world.get::<MeshMaterial3d<StandardMaterial>>(entity).unwrap();

                return asJsObject(vec![
                    ("material", meshMaterialComponent.0.intoJs().into()),
                ]);
            }
            // id if id == TypeId::of::<OutlineStencil>() => {
            //     let outlineComponent = world.get::<OutlineStencil>(entity).unwrap();
            //
            //     return asJsObject(vec![
            //         ("enabled", outlineComponent.enabled.into()),
            //         ("offset", outlineComponent.offset.into()),
            //     ]);
            // }

            // id if id == TypeId::of::<OutlineVolume>() => {
            //     let outlineComponent = world.get::<OutlineVolume>(entity).unwrap();
            //
            //     return asJsObject(vec![
            //         ("colour", outlineComponent.colour.intoJs().into()),
            //         ("width", outlineComponent.width.into()),
            //         ("visible", outlineComponent.visible.into()),
            //     ]);
            // }
            id if id == TypeId::of::<RayCastPickable>() => {
                // Has no properties?
                return Object::new();
            }
            id if id == TypeId::of::<RotationCamera>() => {
                // Has no properties?
                return Object::new();
            }
            id if id == TypeId::of::<Sprite>() => {
                let spriteComponent = world.get::<Sprite>(entity).unwrap();

                return asJsObject(vec![
                    ("image", spriteComponent.image.intoJs().into()),
                    ("textureAtlas", spriteComponent.texture_atlas.intoJs().into()),
                    ("colour", spriteComponent.color.intoJs().into()),
                    ("flip", asJsObject(vec![
                        ("x", spriteComponent.flip_x.into()),
                        ("y", spriteComponent.flip_y.into()),
                    ]).into()),
                    ("customSize", spriteComponent.custom_size.intoJs().into()),
                    ("position", spriteComponent.rect.intoJs().into()),
                    ("align", spriteComponent.anchor.intoJs().into()),
                    ("imageMode", spriteComponent.image_mode.intoJs().into()),
                ]);
            }
            id if id == TypeId::of::<Transform>() => {
                let transformComponent = world.get::<Transform>(entity).unwrap();

                return asJsObject(vec![
                    ("position", transformComponent.translation.intoJs().into()),
                    ("rotation", transformComponent.rotation.intoJs().into()),
                    ("scale", transformComponent.scale.intoJs().into()),
                ]);
            }
            id if id == TypeId::of::<Visibility>() => {
                let visibilityComponent = world.get::<Visibility>(entity).unwrap();

                return visibilityComponent.intoJs().into();
            }
            _ => Object::new(),
        }
    }
}
