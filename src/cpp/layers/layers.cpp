#include "layers.h"
#include <mbgl/style/layers/symbol_layer.hpp>
#include <mbgl/style/property_value.hpp>
#include <mbgl/style/expression/image.hpp>
#include <mbgl/style/types.hpp>
#include <memory>
#include <string>

namespace mln::bridge::style::layers {
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
