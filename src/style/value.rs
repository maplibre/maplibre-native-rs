//! Builds C++ `StyleValue` trees from `serde_json::Value`.

use cxx::UniquePtr;
use serde_json::Value;

use crate::bridge::style_value::{self, StyleValue};
use crate::style::StyleError;

/// Builds an owned `StyleValue` tree mirroring the given JSON value.
pub(crate) fn build_style_value(value: &Value) -> Result<UniquePtr<StyleValue>, StyleError> {
    Ok(match value {
        Value::Null => style_value::make_null(),
        Value::Bool(b) => style_value::make_bool(*b),
        Value::Number(n) => {
            let Some(f) = n.as_f64().filter(|f| f.is_finite()) else {
                return Err(StyleError::JsonNumber(n.to_string()));
            };
            style_value::make_number(f)
        }
        Value::String(s) => style_value::make_string(s),
        Value::Array(items) => {
            let mut arr = style_value::make_array();
            for item in items {
                style_value::array_push(arr.pin_mut(), build_style_value(item)?);
            }
            arr
        }
        Value::Object(map) => {
            let mut obj = style_value::make_object();
            for (key, child) in map {
                style_value::object_insert(obj.pin_mut(), key, build_style_value(child)?);
            }
            obj
        }
    })
}
