#pragma once

#include <mbgl/style/conversion.hpp>
#include <mbgl/style/conversion_impl.hpp>
#include <mbgl/style/layer.hpp>

#include <cmath>
#include <cstdint>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

#include "rust/cxx.h"

namespace mln::bridge {

// JSON-like value tree consumed by MapLibre's style-spec conversion layer.
class StyleValue {
public:
  enum class Kind : std::uint8_t { Null, Bool, Number, String, Array, Object };

  using Array = std::vector<std::unique_ptr<StyleValue>>;
  using Object = std::map<std::string, std::unique_ptr<StyleValue>>;

  StyleValue() : storage_(std::monostate{}) {}
  explicit StyleValue(bool b) : storage_(b) {}
  explicit StyleValue(double n) : storage_(n) {}
  explicit StyleValue(std::string s) : storage_(std::move(s)) {}
  explicit StyleValue(Array a) : storage_(std::move(a)) {}
  explicit StyleValue(Object o) : storage_(std::move(o)) {}

  void push_back(std::unique_ptr<StyleValue> child) {
    std::get<Array>(storage_).push_back(std::move(child));
  }
  void insert(std::string key, std::unique_ptr<StyleValue> child) {
    std::get<Object>(storage_).emplace(std::move(key), std::move(child));
  }

  Kind kind() const noexcept { return static_cast<Kind>(storage_.index()); }

  bool boolean() const { return std::get<bool>(storage_); }
  double number() const { return std::get<double>(storage_); }
  const std::string &str() const { return std::get<std::string>(storage_); }
  const Array &array() const { return std::get<Array>(storage_); }
  const Object &object() const { return std::get<Object>(storage_); }

private:
  // Variant order must match `Kind` so `storage_.index()` == `kind()`.
  using Storage =
      std::variant<std::monostate, bool, double, std::string, Array, Object>;
  static_assert(static_cast<std::size_t>(Kind::Null) == 0);
  static_assert(static_cast<std::size_t>(Kind::Bool) == 1);
  static_assert(static_cast<std::size_t>(Kind::Number) == 2);
  static_assert(static_cast<std::size_t>(Kind::String) == 3);
  static_assert(static_cast<std::size_t>(Kind::Array) == 4);
  static_assert(static_cast<std::size_t>(Kind::Object) == 5);

  Storage storage_;
};

// Factory wrappers exposed to the cxx bridge.
std::unique_ptr<StyleValue> make_null();
std::unique_ptr<StyleValue> make_bool(bool b);
std::unique_ptr<StyleValue> make_number(double n);
std::unique_ptr<StyleValue> make_string(rust::Str s);
std::unique_ptr<StyleValue> make_array();
void array_push(StyleValue &arr, std::unique_ptr<StyleValue> child);
std::unique_ptr<StyleValue> make_object();
void object_insert(StyleValue &obj, rust::Str key,
                   std::unique_ptr<StyleValue> child);

// Parses a style-spec `Layer` object.
std::unique_ptr<mbgl::style::Layer>
layer_from_value(const StyleValue &value, rust::String &error_message);

} // namespace mln::bridge

namespace mbgl::style::conversion {

// Lets MapLibre's conversion layer read from `StyleValue`.
template <> class ConversionTraits<const mln::bridge::StyleValue *> {
  using T = const mln::bridge::StyleValue *;
  using Kind = mln::bridge::StyleValue::Kind;

public:
  static bool isUndefined(T value) {
    return value == nullptr || value->kind() == Kind::Null;
  }

  static bool isArray(T value) {
    return value != nullptr && value->kind() == Kind::Array;
  }

  static std::size_t arrayLength(T value) { return value->array().size(); }

  static T arrayMember(T value, std::size_t i) {
    return value->array()[i].get();
  }

  static bool isObject(T value) {
    return value != nullptr && value->kind() == Kind::Object;
  }

  static std::optional<T> objectMember(T value, const char *name) {
    const auto &obj = value->object();
    auto it = obj.find(name);
    if (it == obj.end()) {
      return {};
    }
    return {it->second.get()};
  }

  template <class Fn> static std::optional<Error> eachMember(T value, Fn &&fn) {
    for (const auto &entry : value->object()) {
      std::optional<Error> result =
          fn({entry.first.data(), entry.first.size()}, entry.second.get());
      if (result) {
        return result;
      }
    }
    return {};
  }

  static std::optional<bool> toBool(T value) {
    if (value->kind() != Kind::Bool)
      return {};
    return value->boolean();
  }

  static std::optional<float> toNumber(T value) {
    if (value->kind() != Kind::Number)
      return {};
    return static_cast<float>(value->number());
  }

  static std::optional<double> toDouble(T value) {
    if (value->kind() != Kind::Number)
      return {};
    return value->number();
  }

  static std::optional<std::string> toString(T value) {
    if (value->kind() != Kind::String)
      return {};
    return value->str();
  }

  static std::optional<Value> toValue(T value) {
    switch (value->kind()) {
    case Kind::Null:
      // `toValue` only yields booleans, numbers, and strings; null is reported
      // via `isUndefined` instead (matching the Node binding's adapter).
      return {};
    case Kind::Bool:
      return {value->boolean()};
    case Kind::Number: {
      const double d = value->number();
      if (std::isfinite(d) && d == std::trunc(d)) {
        if (d >= 0.0 && d < 18446744073709551616.0 /* 2^64 */) {
          return {static_cast<std::uint64_t>(d)};
        }
        if (d >= -9223372036854775808.0 /* -2^63 */ &&
            d < 9223372036854775808.0 /* 2^63 */) {
          return {static_cast<std::int64_t>(d)};
        }
      }
      return {d};
    }
    case Kind::String:
      return {value->str()};
    case Kind::Array:
    case Kind::Object:
      return {};
    }
    return {};
  }

  static std::optional<GeoJSON> toGeoJSON(T, Error &error) {
    error = {"GeoJSON conversion from StyleValue is not implemented"};
    return {};
  }
};

} // namespace mbgl::style::conversion
