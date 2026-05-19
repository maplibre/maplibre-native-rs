#include <memory>

#include <mbgl/style/types.hpp>
#include <mbgl/util/color.hpp>

#include "rust/cxx.h"

namespace mbgl::style {
    class CircleLayer;
    class FillLayer;
    class LineLayer;
    class SymbolLayer;
}

namespace mln::bridge::style::layers {
    std::unique_ptr<mbgl::style::CircleLayer> create_circle_layer(rust::Str layer_id, rust::Str source_id);
    void setCircleColor(const std::unique_ptr<mbgl::style::CircleLayer>& layer, const mbgl::Color& color);
    void setCircleOpacity(const std::unique_ptr<mbgl::style::CircleLayer>& layer, float opacity);
    void setCircleRadius(const std::unique_ptr<mbgl::style::CircleLayer>& layer, float radius);

    std::unique_ptr<mbgl::style::FillLayer> create_fill_layer(rust::Str layer_id, rust::Str source_id);
    void setFillColor(const std::unique_ptr<mbgl::style::FillLayer>& layer, const mbgl::Color& color);
    void setFillOpacity(const std::unique_ptr<mbgl::style::FillLayer>& layer, float opacity);
    void setFillOutlineColor(const std::unique_ptr<mbgl::style::FillLayer>& layer, const mbgl::Color& color);

    std::unique_ptr<mbgl::style::LineLayer> create_line_layer(rust::Str layer_id, rust::Str source_id);
    void setLineColor(const std::unique_ptr<mbgl::style::LineLayer>& layer, const mbgl::Color& color);
    void setLineOpacity(const std::unique_ptr<mbgl::style::LineLayer>& layer, float opacity);
    void setLineWidth(const std::unique_ptr<mbgl::style::LineLayer>& layer, float width);

    std::unique_ptr<mbgl::style::SymbolLayer> create_symbol_layer(rust::Str layer_id, rust::Str source_id);
    void setIconImage(const std::unique_ptr<mbgl::style::SymbolLayer>& layer, rust::Str image_id);
    void setIconAnchor(const std::unique_ptr<mbgl::style::SymbolLayer>& layer, mbgl::style::SymbolAnchorType anchor);
}
