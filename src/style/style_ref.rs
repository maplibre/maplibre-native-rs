use image::DynamicImage;

use crate::bridge::ffi;
use crate::{AnyLayer, ImageId, ImageRenderer, Layer, LayerId, Source, SourceId, StyleError};

/// A mutable reference to the renderer's current map style.
#[derive(Debug)]
pub struct StyleRef<'a, S> {
    image_renderer: &'a mut ImageRenderer<S>,
}

impl<'a, S> StyleRef<'a, S> {
    pub(crate) fn new(image_renderer: &'a mut ImageRenderer<S>) -> Self {
        Self { image_renderer }
    }

    /// Adds an image to the style with the given ID, pixel ratio, and options.
    ///
    /// Pass `true` for `signed_distance_field` to register the image as an SDF (signed
    /// distance field) icon; pass `false` for a regular bitmap icon.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the image.
    pub fn add_image(
        &mut self,
        id: impl AsRef<str>,
        image: &DynamicImage,
        pixel_ratio: f32,
        signed_distance_field: bool,
    ) -> Result<ImageId, StyleError> {
        use image::EncodableLayout;
        let id = id.as_ref();
        let image = image.to_rgba8();
        self.image_renderer.instance.pin_mut().style_add_image(
            id,
            image.as_bytes(),
            ffi::Size { width: image.width(), height: image.height() },
            pixel_ratio,
            signed_distance_field,
        )?;
        Ok(ImageId::new(id.to_owned()))
    }

    /// Removes an image from the style by ID.
    ///
    /// No-op if `id` does not match an existing image.
    pub fn remove_image(&mut self, id: impl AsRef<str>) {
        self.image_renderer.instance.pin_mut().style_remove_image(id.as_ref());
    }

    /// Adds a source to the current map style and returns its stable source ID.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the source.
    pub fn add_source<T: Source>(&mut self, source: T) -> Result<SourceId, StyleError> {
        let source_id = SourceId::new(source.source_id().to_owned());
        self.image_renderer.instance.pin_mut().style_add_source(source.into_source())?;
        Ok(source_id)
    }

    /// Adds a new layer and returns its stable layer ID.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the layer.
    pub fn add_layer<T: Layer>(&mut self, layer: T) -> Result<LayerId, StyleError> {
        let layer_id = LayerId::new(layer.layer_id().to_owned());
        self.add_layer_inner(layer.into_layer(), None)?;
        Ok(layer_id)
    }

    /// Adds a new layer before an existing layer.
    ///
    /// `before_layer` is the ID of an existing layer; the new layer is inserted
    /// directly below it. If `before_layer` does not match any existing layer,
    /// the new layer is appended to the end of the style.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the layer.
    pub fn add_layer_before<T: Layer>(
        &mut self,
        layer: T,
        before_layer: impl AsRef<str>,
    ) -> Result<LayerId, StyleError> {
        let layer_id = LayerId::new(layer.layer_id().to_owned());
        self.add_layer_inner(layer.into_layer(), Some(before_layer.as_ref()))?;
        Ok(layer_id)
    }

    fn add_layer_inner(
        &mut self,
        layer: cxx::UniquePtr<ffi::CxxLayer>,
        before_id: Option<&str>,
    ) -> Result<(), StyleError> {
        // The C++ bridge encodes "append" as an empty string; that detail is
        // contained here so the public API can use separate append/before methods.
        let before_id = before_id.unwrap_or_default();
        self.image_renderer.instance.pin_mut().style_add_layer(layer, before_id)?;
        Ok(())
    }

    /// Removes a layer from the current map style by ID and returns it.
    ///
    /// Returns `None` if `layer_id` does not match an existing layer.
    pub fn remove_layer(&mut self, layer_id: impl AsRef<str>) -> Option<AnyLayer> {
        AnyLayer::from_layer_ptr(
            self.image_renderer.instance.pin_mut().style_remove_layer(layer_id.as_ref()),
        )
    }

    /// Removes a source from the current map style by ID.
    ///
    /// No-op if `source_id` does not match an existing source.
    pub fn remove_source(&mut self, source_id: impl AsRef<str>) {
        self.image_renderer.instance.pin_mut().style_remove_source(source_id.as_ref());
    }
}
