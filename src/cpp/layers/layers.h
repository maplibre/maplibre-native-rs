#include <memory>
#include "rust/cxx.h"

namespace mbgl::style {
    class SymbolLayer;
}

namespace mln::bridge::style::layers {
    std::unique_ptr<mbgl::style::SymbolLayer>
    create_symbol_layer(rust::Str layer_id, rust::Str source_id);
}
