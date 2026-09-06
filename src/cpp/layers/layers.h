#include <memory>

#include <mln/style/types.hpp>
#include <mln/util/color.hpp>

#include "rust/cxx.h"

namespace mln::style {
    class Layer;
    class CircleLayer;
    class FillLayer;
    class LineLayer;
    class SymbolLayer;
}

namespace mln::bridge::style::layers {
    // Upcasts derived `mln::style::Layer` handles to the base type so that
    // `Style::addLayer(unique_ptr<Layer>, ...)` can be invoked through a single
    // bridge function regardless of the concrete layer type.
    std::unique_ptr<mln::style::Layer> circle_into_layer(std::unique_ptr<mln::style::CircleLayer> layer);
    std::unique_ptr<mln::style::Layer> fill_into_layer(std::unique_ptr<mln::style::FillLayer> layer);
    std::unique_ptr<mln::style::Layer> line_into_layer(std::unique_ptr<mln::style::LineLayer> layer);
    std::unique_ptr<mln::style::Layer> symbol_into_layer(std::unique_ptr<mln::style::SymbolLayer> layer);

    rust::String layer_id(const std::unique_ptr<mln::style::Layer>& layer);
    rust::String layer_type(const std::unique_ptr<mln::style::Layer>& layer);

    std::unique_ptr<mln::style::CircleLayer> try_into_circle(std::unique_ptr<mln::style::Layer> layer);
    std::unique_ptr<mln::style::FillLayer> try_into_fill(std::unique_ptr<mln::style::Layer> layer);
    std::unique_ptr<mln::style::LineLayer> try_into_line(std::unique_ptr<mln::style::Layer> layer);
    std::unique_ptr<mln::style::SymbolLayer> try_into_symbol(std::unique_ptr<mln::style::Layer> layer);

    std::unique_ptr<mln::style::CircleLayer> create_circle_layer(rust::Str layer_id, rust::Str source_id);
    void setCircleColor(const std::unique_ptr<mln::style::CircleLayer>& layer, const mln::Color& color);
    void setCircleOpacity(const std::unique_ptr<mln::style::CircleLayer>& layer, float opacity);
    void setCircleRadius(const std::unique_ptr<mln::style::CircleLayer>& layer, float radius);
    void setCircleStrokeColor(const std::unique_ptr<mln::style::CircleLayer>& layer, const mln::Color& color);
    void setCircleStrokeOpacity(const std::unique_ptr<mln::style::CircleLayer>& layer, float opacity);
    void setCircleStrokeWidth(const std::unique_ptr<mln::style::CircleLayer>& layer, float width);

    std::unique_ptr<mln::style::FillLayer> create_fill_layer(rust::Str layer_id, rust::Str source_id);
    void setFillColor(const std::unique_ptr<mln::style::FillLayer>& layer, const mln::Color& color);
    void setFillOpacity(const std::unique_ptr<mln::style::FillLayer>& layer, float opacity);
    void setFillOutlineColor(const std::unique_ptr<mln::style::FillLayer>& layer, const mln::Color& color);

    std::unique_ptr<mln::style::LineLayer> create_line_layer(rust::Str layer_id, rust::Str source_id);
    void setLineColor(const std::unique_ptr<mln::style::LineLayer>& layer, const mln::Color& color);
    void setLineCap(const std::unique_ptr<mln::style::LineLayer>& layer, mln::style::LineCapType cap);
    void setLineJoin(const std::unique_ptr<mln::style::LineLayer>& layer, mln::style::LineJoinType join);
    void setLineOpacity(const std::unique_ptr<mln::style::LineLayer>& layer, float opacity);
    void setLineWidth(const std::unique_ptr<mln::style::LineLayer>& layer, float width);

    std::unique_ptr<mln::style::SymbolLayer> create_symbol_layer(rust::Str layer_id, rust::Str source_id);
    void setIconImage(const std::unique_ptr<mln::style::SymbolLayer>& layer, rust::Str image_id);
    void setIconAnchor(const std::unique_ptr<mln::style::SymbolLayer>& layer, mln::style::SymbolAnchorType anchor);
}
