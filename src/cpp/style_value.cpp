#include "style_value.h"

#include <mbgl/style/conversion/layer.hpp>

#include <string>
#include <utility>

namespace mln::bridge {

std::unique_ptr<StyleValue> make_null() {
    return std::make_unique<StyleValue>();
}

std::unique_ptr<StyleValue> make_bool(bool b) {
    return std::make_unique<StyleValue>(b);
}

std::unique_ptr<StyleValue> make_number(double n) {
    return std::make_unique<StyleValue>(n);
}

std::unique_ptr<StyleValue> make_string(rust::Str s) {
    return std::make_unique<StyleValue>(std::string(s));
}

std::unique_ptr<StyleValue> make_array() {
    return std::make_unique<StyleValue>(StyleValue::Array{});
}

void array_push(StyleValue& arr, std::unique_ptr<StyleValue> child) {
    arr.push_back(std::move(child));
}

std::unique_ptr<StyleValue> make_object() {
    return std::make_unique<StyleValue>(StyleValue::Object{});
}

void object_insert(StyleValue& obj, rust::Str key, std::unique_ptr<StyleValue> child) {
    obj.insert(std::string(key), std::move(child));
}

std::unique_ptr<mbgl::style::Layer> layer_from_value(const StyleValue& value, rust::String& error_message) {
    mbgl::style::conversion::Error error;
    auto result = mbgl::style::conversion::Converter<std::unique_ptr<mbgl::style::Layer>>()(
        mbgl::style::conversion::Convertible(&value), error);
    if (!result) {
        error_message = rust::String(error.message);
        return nullptr;
    }
    return std::move(*result);
}

} // namespace mln::bridge
