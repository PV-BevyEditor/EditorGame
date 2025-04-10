use std::sync::{Arc, Mutex};

use transform_gizmo_bevy::{GizmoResult, GizmoTransform};
use wasm_bindgen::prelude::*;
use bevy::prelude::*;
use js_sys::{Array, Function, Object};

struct HistoryActionQueue {
    actions: Vec<(HistoryActionType, )>,
}

enum HistoryActionType {
    Undo,
    Redo,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = communication)]
    fn getCallbacks(callbackType: &str) -> Array;

    #[wasm_bindgen(js_namespace = historyList)]
    fn dirtyAdd(location: &str, undoAction: &Function, redoAction: &Function, display: &str, changeType: &str);
}

pub fn consoleLog(s: &str) {
    log(s);
}

pub fn triggerInterfaceCallbacks(callbackType: &str, info: Vec<Object>) {
    let callbacks = getCallbacks(callbackType);
    let arr: Array = Array::new_with_length(info.len() as u32);
    
    for (i, obj) in info.iter().enumerate() {
        arr.set(i as u32, JsValue::from(obj));
    }
    for i in 0..callbacks.length() {
        Function::from(callbacks.get(i)).call1(&JsValue::NULL, &arr).unwrap();
    }
}

fn performUndo(_entity: &'static Entity, transform: Arc<Mutex<&mut Transform>>, result: &'static GizmoResult) {
    match result {
        GizmoResult::Translation { delta: _, total } => {
            transform.lock().unwrap().translation -= Vec3::new(total.x as f32, total.y as f32, total.z as f32);
        },
        GizmoResult::Rotation { axis, delta: _, total, is_view_axis: _ } => {
            transform.lock().unwrap().rotation *= Quat::from_axis_angle(Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32), -total as f32);
        },
        GizmoResult::Scale { total } => {
            transform.lock().unwrap().scale /= Vec3::new(total.x as f32, total.y as f32, total.z as f32);
        },
        _ => {},
    }
}
fn performRedo(_entity: &'static Entity, transform: Arc<Mutex<&mut Transform>>, result: &'static GizmoResult) {
    match result {
        GizmoResult::Translation { delta: _, total } => {
            transform.lock().unwrap().translation += Vec3::new(total.x as f32, total.y as f32, total.z as f32);
        },
        GizmoResult::Rotation { axis, delta: _, total, is_view_axis: _ } => {
            transform.lock().unwrap().rotation *= Quat::from_axis_angle(Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32), *total as f32);
        },
        GizmoResult::Scale { total } => {
            transform.lock().unwrap().scale *= Vec3::new(total.x as f32, total.y as f32, total.z as f32);
        },
        _ => {},
    }
}

pub fn addToHistory(gizmoTransform: &'static GizmoTransform, transform: &'static mut Transform, display: &str) {
    let transform = Arc::new(Mutex::new(transform));
    let cloneForUndo = Arc::clone(&transform);
    let cloneForRedo = Arc::clone(&transform);

    dirtyAdd(
        "viewport", 
        Closure::wrap(Box::new(move || {
            performUndo(&gizmoTransform.0, cloneForUndo.clone(), &gizmoTransform.1);
        }) as Box<dyn FnMut()>).as_ref().unchecked_ref(),
        Closure::wrap(Box::new(move || {
            performRedo(&gizmoTransform.0, cloneForRedo.clone(), &gizmoTransform.1);
        }) as Box<dyn FnMut()>).as_ref().unchecked_ref(),
        display,
        "property"
    );
}
