use crate::renderer::bridge::layers;
use cxx::UniquePtr;

pub struct SymbolLayer {
    layer: UniquePtr<layers::SymbolLayer>,
}

impl SymbolLayer {
    /// Create a new symbol layer with the given layer and source IDs.
    pub fn new<S: super::StyleSourceRef>(layer_id: &str, source: &S) -> Self {
        Self {
            layer: layers::create_symbol_layer(layer_id, source.source_id()),
        }
    }

    pub(crate) fn into_inner(self) -> UniquePtr<layers::SymbolLayer> {
        self.layer
    }
}

impl From<SymbolLayer> for super::StyleLayer {
    fn from(value: SymbolLayer) -> Self {
        Self::Symbol(value)
    }
}
