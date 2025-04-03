use bevy::{
    color::Color,
    prelude::*, 
    render::view::RenderLayers,
};
use bevy_mod_outline::{
    OutlineStencil,
    OutlineVolume,
};
use transform_gizmo_bevy::prelude::*;
use crate::{
    EditorConfiguration,
    RotationCamera,
};
#[cfg(target_arch = "wasm32")]
use crate::consoleLog;

pub fn setup(
    mut configStore: ResMut<GizmoConfigStore>,
    mut commands: Commands,
) {
    configStore.config_mut::<DefaultGizmoConfigGroup>().0.render_layers = RenderLayers::layer(0);
    commands.spawn(EditorConfiguration::default());

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3., 5., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        RotationCamera,
        RayCastPickable,
        GizmoCamera,
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

    #[cfg(target_arch = "wasm32")]
    consoleLog("Logged from bevy game setup!");
}

pub fn setupDynamicAssets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
        
    let mesh = meshes.add(Mesh::from(Capsule3d { radius: 3., half_length: 3. }));

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
            visible: false,
        },
        RayCastPickable,
        GizmoTarget::default(),
    ));
}
