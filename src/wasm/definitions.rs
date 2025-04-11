use js_sys::{Array, Function, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = communication)]
    fn getCallbacks(callbackType: &str) -> Array;
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
