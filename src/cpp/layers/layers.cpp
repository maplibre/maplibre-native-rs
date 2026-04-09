#include "layers.h"
#include <mbgl/style/layers/symbol_layer.hpp>
#include <memory>
#include <string>

namespace mln::bridge::style::layers {
    std::unique_ptr<mbgl::style::SymbolLayer>
    create_symbol_layer(rust::Str layer_id, rust::Str source_id) {
        return std::make_unique<mbgl::style::SymbolLayer>(
            std::string(layer_id),
            std::string(source_id));
    }
}
