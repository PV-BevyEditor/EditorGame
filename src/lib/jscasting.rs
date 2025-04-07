use bevy::{asset::{Asset, Handle}, color::{Color, LinearRgba}, math::Vec2, prelude::*, reflect::Enum, sprite::{Anchor, TextureAtlas}};
use js_sys::{JsString, Object, Reflect};
use wasm_bindgen::JsValue;

pub fn asJsObject(values: Vec<(&str, JsValue)>) -> Object {
    let obj = Object::new();
    
    for (key, value) in values {
        Reflect::set(&obj, &JsString::from(key), &value.into()).unwrap();
    }
    
    return obj;
}

pub trait IntoJs {
    fn intoJs(&self) -> Object;
}

impl IntoJs for Color {
    fn intoJs(&self) -> Object {
        let colour = LinearRgba::from(*self);
        let colourObj = Object::new();

        Reflect::set(&colourObj, &JsString::from("r"), &JsValue::from_f64(colour.red as f64)).unwrap();
        Reflect::set(&colourObj, &JsString::from("g"), &JsValue::from_f64(colour.green as f64)).unwrap();
        Reflect::set(&colourObj, &JsString::from("b"), &JsValue::from_f64(colour.blue as f64)).unwrap();
        Reflect::set(&colourObj, &JsString::from("a"), &JsValue::from_f64(colour.alpha as f64)).unwrap();

        return colourObj;
    }
}

impl IntoJs for TextureAtlas {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("atlasLayout"), &JsValue::from(self.layout.id().to_string())).unwrap();
        Reflect::set(&obj, &JsString::from("textureAtlasId"), &JsValue::from(self.index)).unwrap();
        return obj;
    }
}

impl IntoJs for Vec2 {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("x"), &JsValue::from(self.x)).unwrap();
        Reflect::set(&obj, &JsString::from("y"), &JsValue::from(self.y)).unwrap();
        return obj;
    }
}

impl IntoJs for Vec3 {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("x"), &JsValue::from(self.x)).unwrap();
        Reflect::set(&obj, &JsString::from("y"), &JsValue::from(self.y)).unwrap();
        Reflect::set(&obj, &JsString::from("z"), &JsValue::from(self.z)).unwrap();
        return obj;
    }
}

impl IntoJs for Quat {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("x"), &JsValue::from(self.x)).unwrap();
        Reflect::set(&obj, &JsString::from("y"), &JsValue::from(self.y)).unwrap();
        Reflect::set(&obj, &JsString::from("z"), &JsValue::from(self.z)).unwrap();
        Reflect::set(&obj, &JsString::from("w"), &JsValue::from(self.w)).unwrap();
        return obj;
    }
}

impl IntoJs for Rect {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("min"), &self.min.intoJs()).unwrap();
        Reflect::set(&obj, &JsString::from("max"), &self.max.intoJs()).unwrap();
        return obj;
    }
}

impl IntoJs for Anchor {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("type"), &JsValue::from(self.variant_name())).unwrap();
        Reflect::set(&obj, &JsString::from("position"), &format!("{:?}", self).into()).unwrap();
        return obj;
    }
}

impl IntoJs for SpriteImageMode {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("type"), &JsValue::from(self.variant_name())).unwrap();
        return obj;
    }
}

impl IntoJs for Visibility {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("visible"), &JsValue::from(self.variant_name())).unwrap();
        return obj;
    }
}





impl<T: Asset> IntoJs for Handle<T> {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        Reflect::set(&obj, &JsString::from("id"), &JsValue::from(self.id().to_string())).unwrap();
        return obj;
    }
}

impl<T: IntoJs> IntoJs for Option<T> {
    fn intoJs(&self) -> Object {
        let obj = Object::new();
        match self {
            Some(value) => Reflect::set(&obj, &JsString::from("value"), &value.intoJs()).unwrap(),
            None => Reflect::set(&obj, &JsString::from("value"), &JsValue::NULL).unwrap(),
        };

        return obj;
    }
}
