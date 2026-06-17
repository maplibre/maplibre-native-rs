#include "style_value.h"

#include <mbgl/style/conversion/geojson.hpp>
#include <mbgl/style/conversion/layer.hpp>
#include <mbgl/style/conversion/source.hpp>

#include <iomanip>
#include <locale>
#include <sstream>
#include <string>
#include <utility>

namespace mln::bridge {
namespace {

// Stringify a `StyleValue` tree to JSON, for `toGeoJSON()` on inline GeoJSON
// (source `data`, `["within"]` / `["distance"]` expressions).
//
// Note: open to a cleaner approach than this JSON round-trip in C++.

// Escapes and quotes a string as a JSON string literal.
void append_json_string(std::ostringstream& out, const std::string& value) {
    out << '"';
    for (unsigned char c : value) {
        switch (c) {
        case '"':
            out << "\\\"";
            break;
        case '\\':
            out << "\\\\";
            break;
        case '\b':
            out << "\\b";
            break;
        case '\f':
            out << "\\f";
            break;
        case '\n':
            out << "\\n";
            break;
        case '\r':
            out << "\\r";
            break;
        case '\t':
            out << "\\t";
            break;
        default:
            if (c < 0x20) {
                out << "\\u" << std::hex << std::setw(4) << std::setfill('0') << static_cast<int>(c)
                    << std::dec << std::setfill(' ');
            } else {
                out << static_cast<char>(c);
            }
            break;
        }
    }
    out << '"';
}

void append_json(std::ostringstream& out, const StyleValue& value) {
    switch (value.kind()) {
    case StyleValue::Kind::Null:
        out << "null";
        break;
    case StyleValue::Kind::Bool:
        out << (value.boolean() ? "true" : "false");
        break;
    case StyleValue::Kind::Number:
        out << std::setprecision(17) << value.number();
        break;
    case StyleValue::Kind::String:
        append_json_string(out, value.str());
        break;
    case StyleValue::Kind::Array: {
        out << '[';
        bool first = true;
        for (const auto& child : value.array()) {
            if (!first) {
                out << ',';
            }
            first = false;
            append_json(out, *child);
        }
        out << ']';
        break;
    }
    case StyleValue::Kind::Object: {
        out << '{';
        bool first = true;
        for (const auto& entry : value.object()) {
            if (!first) {
                out << ',';
            }
            first = false;
            append_json_string(out, entry.first);
            out << ':';
            append_json(out, *entry.second);
        }
        out << '}';
        break;
    }
    }
}

std::string stringify_style_value(const StyleValue& value) {
    std::ostringstream json;
    // Force a locale-independent decimal point ('.') so numbers stay valid JSON.
    json.imbue(std::locale::classic());
    append_json(json, value);
    return json.str();
}

} // namespace

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

std::unique_ptr<mbgl::style::Source> source_from_value(rust::Str id,
                                                       const StyleValue& value,
                                                       rust::String& error_message) {
    mbgl::style::conversion::Error error;
    auto result = mbgl::style::conversion::Converter<std::unique_ptr<mbgl::style::Source>>()(
        mbgl::style::conversion::Convertible(&value), error, std::string(id));
    if (!result) {
        error_message = rust::String(error.message);
        return nullptr;
    }
    return std::move(*result);
}

std::optional<mbgl::GeoJSON> style_value_to_geojson(const StyleValue& value,
                                                    mbgl::style::conversion::Error& error) {
    try {
        return mbgl::style::conversion::parseGeoJSON(stringify_style_value(value), error);
    } catch (const std::exception& ex) {
        error.message = ex.what();
        return std::nullopt;
    } catch (...) {
        error.message = "failed to parse GeoJSON";
        return std::nullopt;
    }
}

} // namespace mln::bridge
