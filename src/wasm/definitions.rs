use wasm_bindgen::prelude::*;
use js_sys::{Array, Function, Object};

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

pub fn addToHistory(undoAction: Box<dyn Fn()>, redoAction: Box<dyn Fn()>, display: &str) {
    dirtyAdd(
        "viewport", 
        Closure::wrap(Box::new(undoAction) as Box<dyn Fn()>).as_ref().unchecked_ref(),
        Closure::wrap(Box::new(redoAction) as Box<dyn Fn()>).as_ref().unchecked_ref(),
        display, 
        "property"
    );
}
