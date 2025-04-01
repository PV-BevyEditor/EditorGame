use bevy::prelude::*;

// Player/User camera that can be moved around using awsdqe, arrow keys, and mouse
#[derive(Component)]
pub struct RotationCamera;


// Components to mark needed usage of 
#[derive(Component)]
pub struct MoveArrows;
#[derive(Component)]
pub struct ScaleArrows;
#[derive(Component)]
pub struct RotateArrows;
