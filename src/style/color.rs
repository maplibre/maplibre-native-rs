/// A color constructed from straight RGBA channels in the `0.0..=1.0` range.
///
/// MapLibre Native stores colors as premultiplied RGBA. Constructors on this
/// type accept straight RGBA and store the premultiplied representation expected
/// by MapLibre Native.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    /// Creates an opaque RGB color from channel values in the `0.0..=1.0` range.
    #[must_use]
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// Creates an RGBA color from channel values in the `0.0..=1.0` range.
    ///
    /// # Panics
    ///
    /// Panics if any channel is outside the `0.0..=1.0` range.
    #[must_use]
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&red)
                && (0.0..=1.0).contains(&green)
                && (0.0..=1.0).contains(&blue)
                && (0.0..=1.0).contains(&alpha),
            "color channels must be in the 0.0..=1.0 range; got rgba({red}, {green}, {blue}, {alpha})",
        );
        Self { r: red * alpha, g: green * alpha, b: blue * alpha, a: alpha }
    }
}

unsafe impl cxx::ExternType for Color {
    type Id = cxx::type_id!("mbgl::Color");
    type Kind = cxx::kind::Trivial;
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn rgba_stores_premultiplied_channels() {
        assert_eq!(Color::rgba(1.0, 0.0, 0.0, 0.5), Color { r: 0.5, g: 0.0, b: 0.0, a: 0.5 });
    }

    #[test]
    fn rgb_stores_opaque_channels() {
        assert_eq!(Color::rgb(1.0, 0.0, 0.0), Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
    }
}
