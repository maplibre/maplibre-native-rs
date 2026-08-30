#include "layers.h"
#include <mln/style/layer.hpp>
#include <mln/style/layers/circle_layer.hpp>
#include <mln/style/layers/fill_layer.hpp>
#include <mln/style/layers/line_layer.hpp>
#include <mln/style/layers/symbol_layer.hpp>
#include <mln/style/expression/image.hpp>
#include <mln/style/property_value.hpp>
#include <mln/style/types.hpp>
#include <memory>
#include <string>
#include <string_view>
#include <utility>

namespace mln::bridge::style::layers {
    std::unique_ptr<mln::style::Layer> circle_into_layer(std::unique_ptr<mln::style::CircleLayer> layer) {
        return layer;
    }

    std::unique_ptr<mln::style::Layer> fill_into_layer(std::unique_ptr<mln::style::FillLayer> layer) {
        return layer;
    }

    std::unique_ptr<mln::style::Layer> line_into_layer(std::unique_ptr<mln::style::LineLayer> layer) {
        return layer;
    }

    std::unique_ptr<mln::style::Layer> symbol_into_layer(std::unique_ptr<mln::style::SymbolLayer> layer) {
        return layer;
    }

    rust::String layer_id(const std::unique_ptr<mln::style::Layer>& layer) {
        return rust::String(layer->getID());
    }

    rust::String layer_type(const std::unique_ptr<mln::style::Layer>& layer) {
        return rust::String(layer->getTypeInfo()->type);
    }

    namespace {

    template <typename Derived>
    std::unique_ptr<Derived> try_downcast(std::unique_ptr<mln::style::Layer> layer, std::string_view type_name) {
        if (!layer || std::string_view(layer->getTypeInfo()->type) != type_name) {
            return nullptr;
        }
        auto* raw = static_cast<Derived*>(layer.release());
        return std::unique_ptr<Derived>(raw);
    }

    } // namespace

    std::unique_ptr<mln::style::CircleLayer> try_into_circle(std::unique_ptr<mln::style::Layer> layer) {
        return try_downcast<mln::style::CircleLayer>(std::move(layer), "circle");
    }

    std::unique_ptr<mln::style::FillLayer> try_into_fill(std::unique_ptr<mln::style::Layer> layer) {
        return try_downcast<mln::style::FillLayer>(std::move(layer), "fill");
    }

    std::unique_ptr<mln::style::LineLayer> try_into_line(std::unique_ptr<mln::style::Layer> layer) {
        return try_downcast<mln::style::LineLayer>(std::move(layer), "line");
    }

    std::unique_ptr<mln::style::SymbolLayer> try_into_symbol(std::unique_ptr<mln::style::Layer> layer) {
        return try_downcast<mln::style::SymbolLayer>(std::move(layer), "symbol");
    }

    std::unique_ptr<mln::style::CircleLayer> create_circle_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mln::style::CircleLayer>(std::string(layer_id), std::string(source_id));
    }

    void setCircleColor(const std::unique_ptr<mln::style::CircleLayer>& layer, const mln::Color& color) {
        layer->setCircleColor(mln::style::PropertyValue(color));
    }

    void setCircleOpacity(const std::unique_ptr<mln::style::CircleLayer>& layer, float opacity) {
        layer->setCircleOpacity(mln::style::PropertyValue(opacity));
    }

    void setCircleRadius(const std::unique_ptr<mln::style::CircleLayer>& layer, float radius) {
        layer->setCircleRadius(mln::style::PropertyValue(radius));
    }

    void setCircleStrokeColor(const std::unique_ptr<mln::style::CircleLayer>& layer, const mln::Color& color) {
        layer->setCircleStrokeColor(mln::style::PropertyValue(color));
    }

    void setCircleStrokeOpacity(const std::unique_ptr<mln::style::CircleLayer>& layer, float opacity) {
        layer->setCircleStrokeOpacity(mln::style::PropertyValue(opacity));
    }

    void setCircleStrokeWidth(const std::unique_ptr<mln::style::CircleLayer>& layer, float width) {
        layer->setCircleStrokeWidth(mln::style::PropertyValue(width));
    }

    std::unique_ptr<mln::style::FillLayer> create_fill_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mln::style::FillLayer>(std::string(layer_id), std::string(source_id));
    }

    void setFillColor(const std::unique_ptr<mln::style::FillLayer>& layer, const mln::Color& color) {
        layer->setFillColor(mln::style::PropertyValue(color));
    }

    void setFillOpacity(const std::unique_ptr<mln::style::FillLayer>& layer, float opacity) {
        layer->setFillOpacity(mln::style::PropertyValue(opacity));
    }

    void setFillOutlineColor(const std::unique_ptr<mln::style::FillLayer>& layer, const mln::Color& color) {
        layer->setFillOutlineColor(mln::style::PropertyValue(color));
    }

    std::unique_ptr<mln::style::LineLayer> create_line_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mln::style::LineLayer>(std::string(layer_id), std::string(source_id));
    }

    void setLineColor(const std::unique_ptr<mln::style::LineLayer>& layer, const mln::Color& color) {
        layer->setLineColor(mln::style::PropertyValue(color));
    }

    void setLineCap(const std::unique_ptr<mln::style::LineLayer>& layer, mln::style::LineCapType cap) {
        layer->setLineCap(mln::style::PropertyValue(cap));
    }

    void setLineJoin(const std::unique_ptr<mln::style::LineLayer>& layer, mln::style::LineJoinType join) {
        layer->setLineJoin(mln::style::PropertyValue(join));
    }

    void setLineOpacity(const std::unique_ptr<mln::style::LineLayer>& layer, float opacity) {
        layer->setLineOpacity(mln::style::PropertyValue(opacity));
    }

    void setLineWidth(const std::unique_ptr<mln::style::LineLayer>& layer, float width) {
        layer->setLineWidth(mln::style::PropertyValue(width));
    }

    std::unique_ptr<mln::style::SymbolLayer> create_symbol_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mln::style::SymbolLayer>(std::string(layer_id), std::string(source_id));
    }

    void setIconImage(const std::unique_ptr<mln::style::SymbolLayer>& layer, rust::Str image_id) {
        layer->setIconImage(mln::style::PropertyValue(mln::style::expression::Image(std::string(image_id))));
    }

    void setIconAnchor(const std::unique_ptr<mln::style::SymbolLayer>& layer, mln::style::SymbolAnchorType anchor) {
        layer->setIconAnchor(mln::style::PropertyValue(anchor));
    }
}
