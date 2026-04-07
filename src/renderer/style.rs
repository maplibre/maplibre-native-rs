use crate::ImageRenderer;

struct Style<'a, S> {
    image_renderer: &'a ImageRenderer<S>,
}

impl<'a, S> Style<'a, S> {
    pub fn new(image_renderer: &'a ImageRenderer<S>) -> Self {
        Self { image_renderer }
    }
}
