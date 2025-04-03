use bevy::prelude::*;

pub struct EditorCameraConfiguration {
    pub cameraSpeed: f32,
}

pub struct EditorSelectionConfiguration {
    pub selectionColour: Color,
    pub highlightColour: Color,
}

pub struct EditorTransformGizmosConfiguration {
    pub rotationVisible: bool,
    pub translationVisible: bool,
    pub scaleVisible: bool,
    pub arcballVisible: bool,

    pub snapAngle: f32,
    pub snapDistance: f32,
    pub snapScale: f32,
}

#[derive(Component)]
pub struct EditorConfiguration {
    pub camera: EditorCameraConfiguration,
    pub selection: EditorSelectionConfiguration,
}

impl Default for EditorConfiguration {
    fn default() -> Self {
        Self {
            camera: EditorCameraConfiguration { 
                cameraSpeed: 5.,
            },
            selection: EditorSelectionConfiguration { 
                selectionColour: Color::linear_rgb(0.2, 0.2, 0.9),
                highlightColour: Color::linear_rgb(1., 1., 1.),
            }
        }
    }
}
