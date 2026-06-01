#include "layers.h"
#include <mbgl/style/layer.hpp>
#include <mbgl/style/layers/circle_layer.hpp>
#include <mbgl/style/layers/fill_layer.hpp>
#include <mbgl/style/layers/line_layer.hpp>
#include <mbgl/style/layers/symbol_layer.hpp>
#include <mbgl/style/expression/image.hpp>
#include <mbgl/style/property_value.hpp>
#include <mbgl/style/types.hpp>
#include <memory>
#include <string>
#include <string_view>
#include <utility>

namespace mln::bridge::style::layers {
    std::unique_ptr<mbgl::style::Layer> circle_into_layer(std::unique_ptr<mbgl::style::CircleLayer> layer) {
        return layer;
    }

    std::unique_ptr<mbgl::style::Layer> fill_into_layer(std::unique_ptr<mbgl::style::FillLayer> layer) {
        return layer;
    }

    std::unique_ptr<mbgl::style::Layer> line_into_layer(std::unique_ptr<mbgl::style::LineLayer> layer) {
        return layer;
    }

    std::unique_ptr<mbgl::style::Layer> symbol_into_layer(std::unique_ptr<mbgl::style::SymbolLayer> layer) {
        return layer;
    }

    rust::String layer_id(const std::unique_ptr<mbgl::style::Layer>& layer) {
        return rust::String(layer->getID());
    }

    rust::String layer_type(const std::unique_ptr<mbgl::style::Layer>& layer) {
        return rust::String(layer->getTypeInfo()->type);
    }

    namespace {

    template <typename Derived>
    std::unique_ptr<Derived> try_downcast(std::unique_ptr<mbgl::style::Layer> layer, std::string_view type_name) {
        if (!layer || std::string_view(layer->getTypeInfo()->type) != type_name) {
            return nullptr;
        }
        auto* raw = static_cast<Derived*>(layer.release());
        return std::unique_ptr<Derived>(raw);
    }

    } // namespace

    std::unique_ptr<mbgl::style::CircleLayer> try_into_circle(std::unique_ptr<mbgl::style::Layer> layer) {
        return try_downcast<mbgl::style::CircleLayer>(std::move(layer), "circle");
    }

    std::unique_ptr<mbgl::style::FillLayer> try_into_fill(std::unique_ptr<mbgl::style::Layer> layer) {
        return try_downcast<mbgl::style::FillLayer>(std::move(layer), "fill");
    }

    std::unique_ptr<mbgl::style::LineLayer> try_into_line(std::unique_ptr<mbgl::style::Layer> layer) {
        return try_downcast<mbgl::style::LineLayer>(std::move(layer), "line");
    }

    std::unique_ptr<mbgl::style::SymbolLayer> try_into_symbol(std::unique_ptr<mbgl::style::Layer> layer) {
        return try_downcast<mbgl::style::SymbolLayer>(std::move(layer), "symbol");
    }

    std::unique_ptr<mbgl::style::CircleLayer> create_circle_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mbgl::style::CircleLayer>(std::string(layer_id), std::string(source_id));
    }

    void setCircleColor(const std::unique_ptr<mbgl::style::CircleLayer>& layer, const mbgl::Color& color) {
        layer->setCircleColor(mbgl::style::PropertyValue(color));
    }

    void setCircleOpacity(const std::unique_ptr<mbgl::style::CircleLayer>& layer, float opacity) {
        layer->setCircleOpacity(mbgl::style::PropertyValue(opacity));
    }

    void setCircleRadius(const std::unique_ptr<mbgl::style::CircleLayer>& layer, float radius) {
        layer->setCircleRadius(mbgl::style::PropertyValue(radius));
    }

    void setCircleStrokeColor(const std::unique_ptr<mbgl::style::CircleLayer>& layer, const mbgl::Color& color) {
        layer->setCircleStrokeColor(mbgl::style::PropertyValue(color));
    }

    void setCircleStrokeOpacity(const std::unique_ptr<mbgl::style::CircleLayer>& layer, float opacity) {
        layer->setCircleStrokeOpacity(mbgl::style::PropertyValue(opacity));
    }

    void setCircleStrokeWidth(const std::unique_ptr<mbgl::style::CircleLayer>& layer, float width) {
        layer->setCircleStrokeWidth(mbgl::style::PropertyValue(width));
    }

    std::unique_ptr<mbgl::style::FillLayer> create_fill_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mbgl::style::FillLayer>(std::string(layer_id), std::string(source_id));
    }

    void setFillColor(const std::unique_ptr<mbgl::style::FillLayer>& layer, const mbgl::Color& color) {
        layer->setFillColor(mbgl::style::PropertyValue(color));
    }

    void setFillOpacity(const std::unique_ptr<mbgl::style::FillLayer>& layer, float opacity) {
        layer->setFillOpacity(mbgl::style::PropertyValue(opacity));
    }

    void setFillOutlineColor(const std::unique_ptr<mbgl::style::FillLayer>& layer, const mbgl::Color& color) {
        layer->setFillOutlineColor(mbgl::style::PropertyValue(color));
    }

    std::unique_ptr<mbgl::style::LineLayer> create_line_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mbgl::style::LineLayer>(std::string(layer_id), std::string(source_id));
    }

    void setLineColor(const std::unique_ptr<mbgl::style::LineLayer>& layer, const mbgl::Color& color) {
        layer->setLineColor(mbgl::style::PropertyValue(color));
    }

    void setLineCap(const std::unique_ptr<mbgl::style::LineLayer>& layer, mbgl::style::LineCapType cap) {
        layer->setLineCap(mbgl::style::PropertyValue(cap));
    }

    void setLineJoin(const std::unique_ptr<mbgl::style::LineLayer>& layer, mbgl::style::LineJoinType join) {
        layer->setLineJoin(mbgl::style::PropertyValue(join));
    }

    void setLineOpacity(const std::unique_ptr<mbgl::style::LineLayer>& layer, float opacity) {
        layer->setLineOpacity(mbgl::style::PropertyValue(opacity));
    }

    void setLineWidth(const std::unique_ptr<mbgl::style::LineLayer>& layer, float width) {
        layer->setLineWidth(mbgl::style::PropertyValue(width));
    }

    std::unique_ptr<mbgl::style::SymbolLayer> create_symbol_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mbgl::style::SymbolLayer>(std::string(layer_id), std::string(source_id));
    }

    void setIconImage(const std::unique_ptr<mbgl::style::SymbolLayer>& layer, rust::Str image_id) {
        layer->setIconImage(mbgl::style::PropertyValue(mbgl::style::expression::Image(std::string(image_id))));
    }

    void setIconAnchor(const std::unique_ptr<mbgl::style::SymbolLayer>& layer, mbgl::style::SymbolAnchorType anchor) {
        layer->setIconAnchor(mbgl::style::PropertyValue(anchor));
    }
}
