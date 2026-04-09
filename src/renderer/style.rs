use image::DynamicImage;

use crate::renderer::bridge::ffi::{self, Size};
use crate::ImageRenderer;

/// The style of the map
#[derive(Debug)]
pub struct Style<'a, S> {
    image_renderer: &'a mut ImageRenderer<S>,
}

impl<'a, S> Style<'a, S> {
    /// get a style reference from the current map
    pub fn get_ref(image_renderer: &'a mut ImageRenderer<S>) -> Self {
        Self { image_renderer }
    }

    /// Apply the style from the url to the map
    pub fn load_url(&mut self, url: &str) {
        ffi::MapRenderer_getStyle_loadURL(self.image_renderer.instance.pin_mut(), url);
    }

    pub fn add_image(&mut self, id: &str, image: &DynamicImage, single_distance_field: bool) {
        self.image_renderer.instance.pin_mut().style_add_image(
            id,
            image.as_bytes(),
            Size::new(super::Width(image.width()), super::Height(image.height())),
            single_distance_field,
        );
    }
}
