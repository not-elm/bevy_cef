pub mod v8_accessor;
mod v8_handler_wrapper;
pub mod v8_interceptor;

use crate::util::v8_accessor::V8DefaultAccessorBuilder;
use crate::util::v8_interceptor::V8DefaultInterceptorBuilder;
use cef::rc::ConvertParam;
use cef::{
    CefStringList, CefStringUserfreeUtf16, CefStringUtf16, ImplV8Value, V8Propertyattribute,
    v8_value_create_array, v8_value_create_bool, v8_value_create_double, v8_value_create_int,
    v8_value_create_null, v8_value_create_object, v8_value_create_string,
};
use cef_dll_sys::_cef_string_utf16_t;
use cef_dll_sys::cef_scheme_options_t::{
    CEF_SCHEME_OPTION_CORS_ENABLED, CEF_SCHEME_OPTION_FETCH_ENABLED, CEF_SCHEME_OPTION_LOCAL,
    CEF_SCHEME_OPTION_SECURE, CEF_SCHEME_OPTION_STANDARD,
};
use std::env::home_dir;
use std::path::PathBuf;

pub const SCHEME_CEF: &str = "cef";

pub const HOST_CEF: &str = "localhost";

pub fn cef_scheme_flags() -> u32 {
    CEF_SCHEME_OPTION_STANDARD as u32
        | CEF_SCHEME_OPTION_SECURE as u32
        | CEF_SCHEME_OPTION_LOCAL as u32
        | CEF_SCHEME_OPTION_CORS_ENABLED as u32
        | CEF_SCHEME_OPTION_FETCH_ENABLED as u32
}

pub fn debug_chromium_libraries_path() -> PathBuf {
    debug_chromium_embedded_framework_dir_path().join("Libraries")
}

pub fn debug_chromium_embedded_framework_dir_path() -> PathBuf {
    debug_cef_path().join("Chromium Embedded Framework.framework")
}

pub fn debug_cef_path() -> PathBuf {
    home_dir().unwrap().join(".local").join("share").join("cef")
}

pub fn debug_render_process_path() -> PathBuf {
    debug_chromium_libraries_path().join("bevy_cef_debug_render_process")
}

pub trait IntoString {
    fn into_string(self) -> String;
}

impl IntoString for CefStringUserfreeUtf16 {
    fn into_string(self) -> String {
        let ptr: *mut _cef_string_utf16_t = self.into_raw();
        CefStringUtf16::from(ptr).to_string()
    }
}

pub fn v8_value_to_json(v8: &cef::V8Value) -> Option<serde_json::Value> {
    if v8.is_bool().is_positive() {
        Some(serde_json::Value::Bool(v8.bool_value().is_positive()))
    } else if v8.is_int().is_positive() {
        Some(serde_json::Value::Number(serde_json::Number::from(
            v8.int_value(),
        )))
    } else if v8.is_double().is_positive() {
        Some(serde_json::Value::Number(
            serde_json::Number::from_f64(v8.double_value()).unwrap(),
        ))
    } else if v8.is_string().is_positive() {
        Some(serde_json::Value::String(v8.string_value().into_string()))
    } else if v8.is_null().is_positive() || v8.is_undefined().is_positive() {
        Some(serde_json::Value::Null)
    } else if v8.is_array().is_positive() {
        let mut array = Vec::new();
        let mut keys = CefStringList::new();
        v8.keys(Some(&mut keys));
        for key in keys.into_iter() {
            if let Some(v) = v8.value_bykey(Some(&key.as_str().into()))
                && let Some(serialized) = v8_value_to_json(&v)
            {
                {
                    array.push(serialized);
                }
            }
        }
        Some(serde_json::Value::Array(array))
    } else if v8.is_object().is_positive() {
        let mut object = serde_json::Map::new();
        let mut keys = CefStringList::new();
        v8.keys(Some(&mut keys));
        for key in keys.into_iter() {
            if let Some(v) = v8.value_bykey(Some(&key.as_str().into()))
                && let Some(serialized) = v8_value_to_json(&v)
            {
                {
                    object.insert(key, serialized);
                }
            }
        }
        Some(serde_json::Value::Object(object))
    } else {
        None
    }
}

pub fn json_to_v8(v: serde_json::Value) -> Option<cef::V8Value> {
    match v {
        serde_json::Value::Null => v8_value_create_null(),
        serde_json::Value::Bool(b) => v8_value_create_bool(b as _),
        serde_json::Value::Number(n) if n.is_i64() => v8_value_create_int(n.as_i64()? as i32),
        serde_json::Value::Number(n) => v8_value_create_double(n.as_f64()?),
        serde_json::Value::String(s) => v8_value_create_string(Some(&s.as_str().into())),
        serde_json::Value::Array(arr) => {
            let v8_array = v8_value_create_array(arr.len() as _)?;
            for (i, item) in arr.into_iter().enumerate() {
                v8_array.set_value_byindex(i as _, json_to_v8(item).as_mut());
            }
            Some(v8_array)
        }
        serde_json::Value::Object(obj) => {
            let v8_object = v8_value_create_object(
                Some(&mut V8DefaultAccessorBuilder::build()),
                Some(&mut V8DefaultInterceptorBuilder::build()),
            )?;
            for (key, value) in obj {
                v8_object.set_value_bykey(
                    Some(&key.as_str().into()),
                    json_to_v8(value).as_mut(),
                    V8Propertyattribute::default(),
                );
            }
            Some(v8_object)
        }
    }
}
