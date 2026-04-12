use crate::renderer::callbacks::{
    camera_did_change_callback, failing_loading_map_callback, finish_rendering_frame_callback,
    void_callback, CameraDidChangeCallback, FailingLoadingMapCallback,
    FinishRenderingFrameCallback, VoidCallback,
};
use cxx::UniquePtr;
use std::fmt::Display;
use std::ops::Sub;

// https://maplibre.org/maplibre-native/docs/book/design/ten-thousand-foot-view.html

/// Enable or disable the internal logging thread
///
/// By default, logs are generated asynchronously except for Error level messages.
/// In crash scenarios, pending async log entries may be lost.
pub fn set_log_thread_enabled(enable: bool) {
    ffi::Log_useLogThread(enable);
}

fn log_from_cpp(severity: ffi::EventSeverity, event: ffi::Event, code: i64, message: &str) {
    #[cfg(feature = "log")]
    match severity {
        ffi::EventSeverity::Debug => log::debug!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity::Info => log::info!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity::Warning => log::warn!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity::Error => log::error!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity { repr } => {
            log::error!("{event:?} (severity={repr}, code={code}) {message}");
        }
    }
}

/// An x value
#[derive(Debug)]
pub struct X(pub f64);

/// An y value
#[derive(Debug)]
pub struct Y(pub f64);

/// A width value
#[derive(Debug)]
pub struct Width(pub u32);

/// A height value
#[derive(Debug)]
pub struct Height(pub u32);

/// A position in screen coordinates
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct ScreenCoordinate {
    x: f64,
    y: f64,
}

impl ScreenCoordinate {
    /// Create a new `ScreenCoordinate` object
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(x: X, y: Y) -> Self {
        Self { x: x.0, y: y.0 }
    }
}

impl Sub for ScreenCoordinate {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

/// A size
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: u32,
    heigth: u32,
}

impl Size {
    /// Create a new size object
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(width: Width, height: Height) -> Self {
        Self { width: width.0, heigth: height.0 }
    }

    /// get width
    #[must_use]
    pub fn width(self) -> u32 {
        self.width
    }

    /// get height
    #[must_use]
    pub fn height(self) -> u32 {
        self.heigth
    }
}

#[cxx::bridge()]
/// FFI bindings for map source operations.
///
/// This module provides C++/Rust interoperability for various source types.
/// Currently supports GeoJSON sources, with extensibility for additional source types.
pub mod sources {
    #[namespace = "mbgl::style"]
    extern "C++" {
        include!("mbgl/style/sources/geojson_source.hpp");
        // Opaque types
        /// A GeoJSON source for MapLibre rendering.
        type GeoJSONSource;
    }

    #[namespace = "mln::bridge::style::sources::geojson"]
    unsafe extern "C++" {
        include!("sources/sources.h");

        /// Creates a new GeoJSON source with the given ID.
        fn create(id: &str) -> UniquePtr<GeoJSONSource>;
        /// Sets a point for the GeoJSON source.
        fn setPoint(source: &UniquePtr<GeoJSONSource>, latitude: f64, longitude: f64);
    }
}

impl std::fmt::Debug for sources::GeoJSONSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeoJSONSource").finish()
    }
}

#[cxx::bridge()]
/// FFI bindings for map layer operations.
///
/// This module provides C++/Rust interoperability for various layer types.
/// Currently supports symbol layers, with extensibility for additional layer types like fill, line, and background layers.
pub mod layers {
    // Must have the same namespace than on the C++ side
    #[namespace = "mbgl::style"]
    /// Symbol anchor position type.
    pub enum SymbolAnchorType {
        /// Center anchor point.
        Center,
        /// Left anchor point.
        Left,
        /// Right anchor point.
        Right,
        /// Top anchor point.
        Top,
        /// Bottom anchor point.
        Bottom,
        /// Top-left anchor point.
        TopLeft,
        /// Top-right anchor point.
        TopRight,
        /// Bottom-left anchor point.
        BottomLeft,
        /// Bottom-right anchor point.
        BottomRight,
    }

    #[namespace = "mbgl::style"]
    extern "C++" {
        include!("mbgl/style/layers/symbol_layer.hpp");
        include!("mbgl/style/types.hpp");
        // Opaque types
        /// A symbol layer for rendering labels and icons on the map.
        type SymbolLayer;

        /// Symbol anchor position type.
        type SymbolAnchorType;
    }

    #[namespace = "mln::bridge::style::layers"]
    unsafe extern "C++" {
        include!("layers/layers.h");

        /// Creates a new symbol layer.
        #[must_use]
        pub(crate) fn create_symbol_layer(
            layer_id: &str,
            source_id: &str,
        ) -> UniquePtr<SymbolLayer>;
        /// Sets the icon image for a layer by image ID.
        fn setIconImage(layer: &UniquePtr<SymbolLayer>, image_id: &str);
        /// Sets the anchor point for layer icons.
        fn setIconAnchor(layer: &UniquePtr<SymbolLayer>, anchor: SymbolAnchorType);
    }
}

impl std::fmt::Debug for layers::SymbolLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolLayer").finish()
    }
}

impl std::fmt::Debug for layers::SymbolAnchorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolAnchorType").finish()
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<ffi::WGPUTextureDimension> for ::wgpu::TextureDimension {
    type Error = ();
    fn try_from(value: ffi::WGPUTextureDimension) -> Result<Self, Self::Error> {
        match value {
            ffi::WGPUTextureDimension::WGPUTextureDimension_Undefined => Err(()),
            ffi::WGPUTextureDimension::WGPUTextureDimension_1D => Ok(::wgpu::TextureDimension::D1),
            ffi::WGPUTextureDimension::WGPUTextureDimension_2D => Ok(::wgpu::TextureDimension::D2),
            ffi::WGPUTextureDimension::WGPUTextureDimension_3D => Ok(::wgpu::TextureDimension::D2),
            _ => Err(()),
        }
    }
}

/// Wraps the C `WGPUTextureUsage` typedef (`uint64_t`).
#[cfg(feature = "wgpu")]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WGPUTextureUsage(pub u64);

#[cfg(feature = "wgpu")]
unsafe impl cxx::ExternType for WGPUTextureUsage {
    type Id = cxx::type_id!("WGPUTextureUsage");
    type Kind = cxx::kind::Trivial;
}

#[cfg(feature = "wgpu")]
impl From<WGPUTextureUsage> for ::wgpu::TextureUsages {
    fn from(v: WGPUTextureUsage) -> Self {
        ::wgpu::TextureUsages::from_bits_truncate(v.0 as u32)
    }
}

/// Wraps the C `WGPUExtent3D` struct. Field `depth_or_array_layers` corresponds
/// to C's `depthOrArrayLayers`.
#[cfg(feature = "wgpu")]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WGPUExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}

#[cfg(feature = "wgpu")]
unsafe impl cxx::ExternType for WGPUExtent3D {
    type Id = cxx::type_id!("WGPUExtent3D");
    type Kind = cxx::kind::Trivial;
}

#[cfg(feature = "wgpu")]
impl From<WGPUExtent3D> for ::wgpu::Extent3d {
    fn from(v: WGPUExtent3D) -> Self {
        ::wgpu::Extent3d {
            width: v.width,
            height: v.height,
            depth_or_array_layers: v.depth_or_array_layers,
        }
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<ffi::WGPUTextureAspect> for ::wgpu::TextureAspect {
    type Error = ();
    fn try_from(value: ffi::WGPUTextureAspect) -> Result<Self, Self::Error> {
        match value {
            ffi::WGPUTextureAspect::WGPUTextureAspect_Undefined => Err(()),
            ffi::WGPUTextureAspect::WGPUTextureAspect_All => Ok(::wgpu::TextureAspect::All),
            ffi::WGPUTextureAspect::WGPUTextureAspect_StencilOnly => {
                Ok(::wgpu::TextureAspect::StencilOnly)
            }
            ffi::WGPUTextureAspect::WGPUTextureAspect_DepthOnly => {
                Ok(::wgpu::TextureAspect::DepthOnly)
            }
            _ => Err(()),
        }
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<ffi::WGPUTextureViewDimension> for ::wgpu::TextureViewDimension {
    type Error = ();
    fn try_from(value: ffi::WGPUTextureViewDimension) -> Result<Self, Self::Error> {
        use ::wgpu::TextureViewDimension as TVD;
        match value {
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_Undefined => Err(()),
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_1D => Ok(TVD::D1),
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_2D => Ok(TVD::D2),
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_2DArray => Ok(TVD::D2Array),
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_Cube => Ok(TVD::Cube),
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_CubeArray => Ok(TVD::CubeArray),
            ffi::WGPUTextureViewDimension::WGPUTextureViewDimension_3D => Ok(TVD::D3),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<ffi::WGPUTextureFormat> for ::wgpu::TextureFormat {
    type Error = ();
    fn try_from(value: ffi::WGPUTextureFormat) -> Result<Self, Self::Error> {
        use ::wgpu::{AstcBlock, AstcChannel, TextureFormat as T};
        use ffi::WGPUTextureFormat as C;
        Ok(match value {
            C::WGPUTextureFormat_Undefined => return Err(()),
            C::WGPUTextureFormat_R8Unorm => T::R8Unorm,
            C::WGPUTextureFormat_R8Snorm => T::R8Snorm,
            C::WGPUTextureFormat_R8Uint => T::R8Uint,
            C::WGPUTextureFormat_R8Sint => T::R8Sint,
            C::WGPUTextureFormat_R16Uint => T::R16Uint,
            C::WGPUTextureFormat_R16Sint => T::R16Sint,
            C::WGPUTextureFormat_R16Float => T::R16Float,
            C::WGPUTextureFormat_RG8Unorm => T::Rg8Unorm,
            C::WGPUTextureFormat_RG8Snorm => T::Rg8Snorm,
            C::WGPUTextureFormat_RG8Uint => T::Rg8Uint,
            C::WGPUTextureFormat_RG8Sint => T::Rg8Sint,
            C::WGPUTextureFormat_R32Float => T::R32Float,
            C::WGPUTextureFormat_R32Uint => T::R32Uint,
            C::WGPUTextureFormat_R32Sint => T::R32Sint,
            C::WGPUTextureFormat_RG16Uint => T::Rg16Uint,
            C::WGPUTextureFormat_RG16Sint => T::Rg16Sint,
            C::WGPUTextureFormat_RG16Float => T::Rg16Float,
            C::WGPUTextureFormat_RGBA8Unorm => T::Rgba8Unorm,
            C::WGPUTextureFormat_RGBA8UnormSrgb => T::Rgba8UnormSrgb,
            C::WGPUTextureFormat_RGBA8Snorm => T::Rgba8Snorm,
            C::WGPUTextureFormat_RGBA8Uint => T::Rgba8Uint,
            C::WGPUTextureFormat_RGBA8Sint => T::Rgba8Sint,
            C::WGPUTextureFormat_BGRA8Unorm => T::Bgra8Unorm,
            C::WGPUTextureFormat_BGRA8UnormSrgb => T::Bgra8UnormSrgb,
            C::WGPUTextureFormat_RGB10A2Uint => T::Rgb10a2Uint,
            C::WGPUTextureFormat_RGB10A2Unorm => T::Rgb10a2Unorm,
            C::WGPUTextureFormat_RG11B10Ufloat => T::Rg11b10Ufloat,
            C::WGPUTextureFormat_RGB9E5Ufloat => T::Rgb9e5Ufloat,
            C::WGPUTextureFormat_RG32Float => T::Rg32Float,
            C::WGPUTextureFormat_RG32Uint => T::Rg32Uint,
            C::WGPUTextureFormat_RG32Sint => T::Rg32Sint,
            C::WGPUTextureFormat_RGBA16Uint => T::Rgba16Uint,
            C::WGPUTextureFormat_RGBA16Sint => T::Rgba16Sint,
            C::WGPUTextureFormat_RGBA16Float => T::Rgba16Float,
            C::WGPUTextureFormat_RGBA32Float => T::Rgba32Float,
            C::WGPUTextureFormat_RGBA32Uint => T::Rgba32Uint,
            C::WGPUTextureFormat_RGBA32Sint => T::Rgba32Sint,
            C::WGPUTextureFormat_Stencil8 => T::Stencil8,
            C::WGPUTextureFormat_Depth16Unorm => T::Depth16Unorm,
            C::WGPUTextureFormat_Depth24Plus => T::Depth24Plus,
            C::WGPUTextureFormat_Depth24PlusStencil8 => T::Depth24PlusStencil8,
            C::WGPUTextureFormat_Depth32Float => T::Depth32Float,
            C::WGPUTextureFormat_Depth32FloatStencil8 => T::Depth32FloatStencil8,
            C::WGPUTextureFormat_BC1RGBAUnorm => T::Bc1RgbaUnorm,
            C::WGPUTextureFormat_BC1RGBAUnormSrgb => T::Bc1RgbaUnormSrgb,
            C::WGPUTextureFormat_BC2RGBAUnorm => T::Bc2RgbaUnorm,
            C::WGPUTextureFormat_BC2RGBAUnormSrgb => T::Bc2RgbaUnormSrgb,
            C::WGPUTextureFormat_BC3RGBAUnorm => T::Bc3RgbaUnorm,
            C::WGPUTextureFormat_BC3RGBAUnormSrgb => T::Bc3RgbaUnormSrgb,
            C::WGPUTextureFormat_BC4RUnorm => T::Bc4RUnorm,
            C::WGPUTextureFormat_BC4RSnorm => T::Bc4RSnorm,
            C::WGPUTextureFormat_BC5RGUnorm => T::Bc5RgUnorm,
            C::WGPUTextureFormat_BC5RGSnorm => T::Bc5RgSnorm,
            C::WGPUTextureFormat_BC6HRGBUfloat => T::Bc6hRgbUfloat,
            C::WGPUTextureFormat_BC6HRGBFloat => T::Bc6hRgbFloat,
            C::WGPUTextureFormat_BC7RGBAUnorm => T::Bc7RgbaUnorm,
            C::WGPUTextureFormat_BC7RGBAUnormSrgb => T::Bc7RgbaUnormSrgb,
            C::WGPUTextureFormat_ETC2RGB8Unorm => T::Etc2Rgb8Unorm,
            C::WGPUTextureFormat_ETC2RGB8UnormSrgb => T::Etc2Rgb8UnormSrgb,
            C::WGPUTextureFormat_ETC2RGB8A1Unorm => T::Etc2Rgb8A1Unorm,
            C::WGPUTextureFormat_ETC2RGB8A1UnormSrgb => T::Etc2Rgb8A1UnormSrgb,
            C::WGPUTextureFormat_ETC2RGBA8Unorm => T::Etc2Rgba8Unorm,
            C::WGPUTextureFormat_ETC2RGBA8UnormSrgb => T::Etc2Rgba8UnormSrgb,
            C::WGPUTextureFormat_EACR11Unorm => T::EacR11Unorm,
            C::WGPUTextureFormat_EACR11Snorm => T::EacR11Snorm,
            C::WGPUTextureFormat_EACRG11Unorm => T::EacRg11Unorm,
            C::WGPUTextureFormat_EACRG11Snorm => T::EacRg11Snorm,
            C::WGPUTextureFormat_ASTC4x4Unorm => {
                T::Astc { block: AstcBlock::B4x4, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC4x4UnormSrgb => {
                T::Astc { block: AstcBlock::B4x4, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC5x4Unorm => {
                T::Astc { block: AstcBlock::B5x4, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC5x4UnormSrgb => {
                T::Astc { block: AstcBlock::B5x4, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC5x5Unorm => {
                T::Astc { block: AstcBlock::B5x5, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC5x5UnormSrgb => {
                T::Astc { block: AstcBlock::B5x5, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC6x5Unorm => {
                T::Astc { block: AstcBlock::B6x5, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC6x5UnormSrgb => {
                T::Astc { block: AstcBlock::B6x5, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC6x6Unorm => {
                T::Astc { block: AstcBlock::B6x6, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC6x6UnormSrgb => {
                T::Astc { block: AstcBlock::B6x6, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC8x5Unorm => {
                T::Astc { block: AstcBlock::B8x5, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC8x5UnormSrgb => {
                T::Astc { block: AstcBlock::B8x5, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC8x6Unorm => {
                T::Astc { block: AstcBlock::B8x6, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC8x6UnormSrgb => {
                T::Astc { block: AstcBlock::B8x6, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC8x8Unorm => {
                T::Astc { block: AstcBlock::B8x8, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC8x8UnormSrgb => {
                T::Astc { block: AstcBlock::B8x8, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC10x5Unorm => {
                T::Astc { block: AstcBlock::B10x5, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC10x5UnormSrgb => {
                T::Astc { block: AstcBlock::B10x5, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC10x6Unorm => {
                T::Astc { block: AstcBlock::B10x6, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC10x6UnormSrgb => {
                T::Astc { block: AstcBlock::B10x6, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC10x8Unorm => {
                T::Astc { block: AstcBlock::B10x8, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC10x8UnormSrgb => {
                T::Astc { block: AstcBlock::B10x8, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC10x10Unorm => {
                T::Astc { block: AstcBlock::B10x10, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC10x10UnormSrgb => {
                T::Astc { block: AstcBlock::B10x10, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC12x10Unorm => {
                T::Astc { block: AstcBlock::B12x10, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC12x10UnormSrgb => {
                T::Astc { block: AstcBlock::B12x10, channel: AstcChannel::UnormSrgb }
            }
            C::WGPUTextureFormat_ASTC12x12Unorm => {
                T::Astc { block: AstcBlock::B12x12, channel: AstcChannel::Unorm }
            }
            C::WGPUTextureFormat_ASTC12x12UnormSrgb => {
                T::Astc { block: AstcBlock::B12x12, channel: AstcChannel::UnormSrgb }
            }
            _ => return Err(()),
        })
    }
}

#[cfg(feature = "wgpu")]
impl From<::wgpu::TextureDimension> for ffi::WGPUTextureDimension {
    fn from(v: ::wgpu::TextureDimension) -> Self {
        match v {
            ::wgpu::TextureDimension::D1 => ffi::WGPUTextureDimension::WGPUTextureDimension_1D,
            ::wgpu::TextureDimension::D2 => ffi::WGPUTextureDimension::WGPUTextureDimension_2D,
            ::wgpu::TextureDimension::D3 => ffi::WGPUTextureDimension::WGPUTextureDimension_3D,
        }
    }
}

#[cfg(feature = "wgpu")]
impl From<::wgpu::TextureUsages> for WGPUTextureUsage {
    fn from(v: ::wgpu::TextureUsages) -> Self {
        WGPUTextureUsage(v.bits() as u64)
    }
}

#[cfg(feature = "wgpu")]
impl From<::wgpu::Extent3d> for WGPUExtent3D {
    fn from(v: ::wgpu::Extent3d) -> Self {
        WGPUExtent3D {
            width: v.width,
            height: v.height,
            depth_or_array_layers: v.depth_or_array_layers,
        }
    }
}

#[cfg(feature = "wgpu")]
impl From<::wgpu::TextureAspect> for ffi::WGPUTextureAspect {
    fn from(v: ::wgpu::TextureAspect) -> Self {
        match v {
            ::wgpu::TextureAspect::All => ffi::WGPUTextureAspect::WGPUTextureAspect_All,
            ::wgpu::TextureAspect::StencilOnly => {
                ffi::WGPUTextureAspect::WGPUTextureAspect_StencilOnly
            }
            ::wgpu::TextureAspect::DepthOnly => ffi::WGPUTextureAspect::WGPUTextureAspect_DepthOnly,
            // Plane variants have no C WebGPU equivalent; fall back to All
            _ => ffi::WGPUTextureAspect::WGPUTextureAspect_All,
        }
    }
}

#[cfg(feature = "wgpu")]
impl From<::wgpu::TextureViewDimension> for ffi::WGPUTextureViewDimension {
    fn from(v: ::wgpu::TextureViewDimension) -> Self {
        use ffi::WGPUTextureViewDimension as C;
        match v {
            ::wgpu::TextureViewDimension::D1 => C::WGPUTextureViewDimension_1D,
            ::wgpu::TextureViewDimension::D2 => C::WGPUTextureViewDimension_2D,
            ::wgpu::TextureViewDimension::D2Array => C::WGPUTextureViewDimension_2DArray,
            ::wgpu::TextureViewDimension::Cube => C::WGPUTextureViewDimension_Cube,
            ::wgpu::TextureViewDimension::CubeArray => C::WGPUTextureViewDimension_CubeArray,
            ::wgpu::TextureViewDimension::D3 => C::WGPUTextureViewDimension_3D,
        }
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<::wgpu::TextureFormat> for ffi::WGPUTextureFormat {
    type Error = ();
    fn try_from(v: ::wgpu::TextureFormat) -> Result<Self, Self::Error> {
        use ::wgpu::{AstcBlock, AstcChannel, TextureFormat as T};
        use ffi::WGPUTextureFormat as C;
        Ok(match v {
            T::R8Unorm => C::WGPUTextureFormat_R8Unorm,
            T::R8Snorm => C::WGPUTextureFormat_R8Snorm,
            T::R8Uint => C::WGPUTextureFormat_R8Uint,
            T::R8Sint => C::WGPUTextureFormat_R8Sint,
            T::R16Uint => C::WGPUTextureFormat_R16Uint,
            T::R16Sint => C::WGPUTextureFormat_R16Sint,
            T::R16Float => C::WGPUTextureFormat_R16Float,
            T::Rg8Unorm => C::WGPUTextureFormat_RG8Unorm,
            T::Rg8Snorm => C::WGPUTextureFormat_RG8Snorm,
            T::Rg8Uint => C::WGPUTextureFormat_RG8Uint,
            T::Rg8Sint => C::WGPUTextureFormat_RG8Sint,
            T::R32Float => C::WGPUTextureFormat_R32Float,
            T::R32Uint => C::WGPUTextureFormat_R32Uint,
            T::R32Sint => C::WGPUTextureFormat_R32Sint,
            T::Rg16Uint => C::WGPUTextureFormat_RG16Uint,
            T::Rg16Sint => C::WGPUTextureFormat_RG16Sint,
            T::Rg16Float => C::WGPUTextureFormat_RG16Float,
            T::Rgba8Unorm => C::WGPUTextureFormat_RGBA8Unorm,
            T::Rgba8UnormSrgb => C::WGPUTextureFormat_RGBA8UnormSrgb,
            T::Rgba8Snorm => C::WGPUTextureFormat_RGBA8Snorm,
            T::Rgba8Uint => C::WGPUTextureFormat_RGBA8Uint,
            T::Rgba8Sint => C::WGPUTextureFormat_RGBA8Sint,
            T::Bgra8Unorm => C::WGPUTextureFormat_BGRA8Unorm,
            T::Bgra8UnormSrgb => C::WGPUTextureFormat_BGRA8UnormSrgb,
            T::Rgb10a2Uint => C::WGPUTextureFormat_RGB10A2Uint,
            T::Rgb10a2Unorm => C::WGPUTextureFormat_RGB10A2Unorm,
            T::Rg11b10Ufloat => C::WGPUTextureFormat_RG11B10Ufloat,
            T::Rgb9e5Ufloat => C::WGPUTextureFormat_RGB9E5Ufloat,
            T::Rg32Float => C::WGPUTextureFormat_RG32Float,
            T::Rg32Uint => C::WGPUTextureFormat_RG32Uint,
            T::Rg32Sint => C::WGPUTextureFormat_RG32Sint,
            T::Rgba16Uint => C::WGPUTextureFormat_RGBA16Uint,
            T::Rgba16Sint => C::WGPUTextureFormat_RGBA16Sint,
            T::Rgba16Float => C::WGPUTextureFormat_RGBA16Float,
            T::Rgba32Float => C::WGPUTextureFormat_RGBA32Float,
            T::Rgba32Uint => C::WGPUTextureFormat_RGBA32Uint,
            T::Rgba32Sint => C::WGPUTextureFormat_RGBA32Sint,
            T::Stencil8 => C::WGPUTextureFormat_Stencil8,
            T::Depth16Unorm => C::WGPUTextureFormat_Depth16Unorm,
            T::Depth24Plus => C::WGPUTextureFormat_Depth24Plus,
            T::Depth24PlusStencil8 => C::WGPUTextureFormat_Depth24PlusStencil8,
            T::Depth32Float => C::WGPUTextureFormat_Depth32Float,
            T::Depth32FloatStencil8 => C::WGPUTextureFormat_Depth32FloatStencil8,
            T::Bc1RgbaUnorm => C::WGPUTextureFormat_BC1RGBAUnorm,
            T::Bc1RgbaUnormSrgb => C::WGPUTextureFormat_BC1RGBAUnormSrgb,
            T::Bc2RgbaUnorm => C::WGPUTextureFormat_BC2RGBAUnorm,
            T::Bc2RgbaUnormSrgb => C::WGPUTextureFormat_BC2RGBAUnormSrgb,
            T::Bc3RgbaUnorm => C::WGPUTextureFormat_BC3RGBAUnorm,
            T::Bc3RgbaUnormSrgb => C::WGPUTextureFormat_BC3RGBAUnormSrgb,
            T::Bc4RUnorm => C::WGPUTextureFormat_BC4RUnorm,
            T::Bc4RSnorm => C::WGPUTextureFormat_BC4RSnorm,
            T::Bc5RgUnorm => C::WGPUTextureFormat_BC5RGUnorm,
            T::Bc5RgSnorm => C::WGPUTextureFormat_BC5RGSnorm,
            T::Bc6hRgbUfloat => C::WGPUTextureFormat_BC6HRGBUfloat,
            T::Bc6hRgbFloat => C::WGPUTextureFormat_BC6HRGBFloat,
            T::Bc7RgbaUnorm => C::WGPUTextureFormat_BC7RGBAUnorm,
            T::Bc7RgbaUnormSrgb => C::WGPUTextureFormat_BC7RGBAUnormSrgb,
            T::Etc2Rgb8Unorm => C::WGPUTextureFormat_ETC2RGB8Unorm,
            T::Etc2Rgb8UnormSrgb => C::WGPUTextureFormat_ETC2RGB8UnormSrgb,
            T::Etc2Rgb8A1Unorm => C::WGPUTextureFormat_ETC2RGB8A1Unorm,
            T::Etc2Rgb8A1UnormSrgb => C::WGPUTextureFormat_ETC2RGB8A1UnormSrgb,
            T::Etc2Rgba8Unorm => C::WGPUTextureFormat_ETC2RGBA8Unorm,
            T::Etc2Rgba8UnormSrgb => C::WGPUTextureFormat_ETC2RGBA8UnormSrgb,
            T::EacR11Unorm => C::WGPUTextureFormat_EACR11Unorm,
            T::EacR11Snorm => C::WGPUTextureFormat_EACR11Snorm,
            T::EacRg11Unorm => C::WGPUTextureFormat_EACRG11Unorm,
            T::EacRg11Snorm => C::WGPUTextureFormat_EACRG11Snorm,
            T::Astc { block: AstcBlock::B4x4, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC4x4Unorm
            }
            T::Astc { block: AstcBlock::B4x4, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC4x4UnormSrgb
            }
            T::Astc { block: AstcBlock::B5x4, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC5x4Unorm
            }
            T::Astc { block: AstcBlock::B5x4, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC5x4UnormSrgb
            }
            T::Astc { block: AstcBlock::B5x5, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC5x5Unorm
            }
            T::Astc { block: AstcBlock::B5x5, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC5x5UnormSrgb
            }
            T::Astc { block: AstcBlock::B6x5, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC6x5Unorm
            }
            T::Astc { block: AstcBlock::B6x5, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC6x5UnormSrgb
            }
            T::Astc { block: AstcBlock::B6x6, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC6x6Unorm
            }
            T::Astc { block: AstcBlock::B6x6, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC6x6UnormSrgb
            }
            T::Astc { block: AstcBlock::B8x5, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC8x5Unorm
            }
            T::Astc { block: AstcBlock::B8x5, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC8x5UnormSrgb
            }
            T::Astc { block: AstcBlock::B8x6, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC8x6Unorm
            }
            T::Astc { block: AstcBlock::B8x6, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC8x6UnormSrgb
            }
            T::Astc { block: AstcBlock::B8x8, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC8x8Unorm
            }
            T::Astc { block: AstcBlock::B8x8, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC8x8UnormSrgb
            }
            T::Astc { block: AstcBlock::B10x5, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC10x5Unorm
            }
            T::Astc { block: AstcBlock::B10x5, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC10x5UnormSrgb
            }
            T::Astc { block: AstcBlock::B10x6, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC10x6Unorm
            }
            T::Astc { block: AstcBlock::B10x6, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC10x6UnormSrgb
            }
            T::Astc { block: AstcBlock::B10x8, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC10x8Unorm
            }
            T::Astc { block: AstcBlock::B10x8, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC10x8UnormSrgb
            }
            T::Astc { block: AstcBlock::B10x10, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC10x10Unorm
            }
            T::Astc { block: AstcBlock::B10x10, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC10x10UnormSrgb
            }
            T::Astc { block: AstcBlock::B12x10, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC12x10Unorm
            }
            T::Astc { block: AstcBlock::B12x10, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC12x10UnormSrgb
            }
            T::Astc { block: AstcBlock::B12x12, channel: AstcChannel::Unorm } => {
                C::WGPUTextureFormat_ASTC12x12Unorm
            }
            T::Astc { block: AstcBlock::B12x12, channel: AstcChannel::UnormSrgb } => {
                C::WGPUTextureFormat_ASTC12x12UnormSrgb
            }
            // Formats with no WebGPU C equivalent (wgpu native extensions)
            _ => return Err(()),
        })
    }
}

#[cxx::bridge()]
/// Resource and configuration options for MapLibre.
pub mod resource_options {

    #[namespace = "mbgl"]
    extern "C++" {
        // Opaque types
        /// Resource configuration options.
        type ResourceOptions;

        // The name must be unique but for some reason this is required
        #[rust_name = "CxxTileServerOptions"]
        type TileServerOptions = super::tile_server_options::TileServerOptions;
    }

    #[namespace = "mln::bridge::resource_options"]
    unsafe extern "C++" {
        include!("resource_options.h");

        #[rust_name = "new"]
        fn new_() -> UniquePtr<ResourceOptions>;

        fn withApiKey(obj: Pin<&mut ResourceOptions>, key: &str);
        fn withAssetPath(obj: Pin<&mut ResourceOptions>, path: &[u8]);
        fn withCachePath(obj: Pin<&mut ResourceOptions>, path: &[u8]);

        fn withMaximumCacheSize(obj: Pin<&mut ResourceOptions>, max_cache_size: u64);
        fn withTileServerOptions(
            obj: Pin<&mut ResourceOptions>,
            tile_server_options: &CxxTileServerOptions,
        );
    }
}

impl std::fmt::Debug for resource_options::ResourceOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceOptions").finish()
    }
}

#[cxx::bridge()]
/// Tile server configuration options.
pub mod tile_server_options {
    #[namespace = "mbgl"]
    extern "C++" {
        // Opaque types
        /// Tile server configuration.
        type TileServerOptions;
    }

    #[namespace = "mln::bridge::tile_server_options"]
    unsafe extern "C++" {
        include!("tile_server_options.h");

        #[rust_name = "new_tile_server_options"]
        fn new_() -> UniquePtr<TileServerOptions>;

        fn withBaseUrl(obj: Pin<&mut TileServerOptions>, path: &[u8]);
        fn withUriSchemeAlias(obj: Pin<&mut TileServerOptions>, path: &[u8]);
        fn withSourceTemplate(
            obj: Pin<&mut TileServerOptions>,
            styleTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withSpritesTemplate(
            obj: Pin<&mut TileServerOptions>,
            spritesTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withGlyphsTemplate(
            obj: Pin<&mut TileServerOptions>,
            glyphsTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withTileTemplate(
            obj: Pin<&mut TileServerOptions>,
            tileTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withApiKeyParameterName(obj: Pin<&mut TileServerOptions>, apiKeyParameterName: &[u8]);
        fn setRequiresApiKey(obj: Pin<&mut TileServerOptions>, apiKeyRequired: bool);
    }
}

impl std::fmt::Debug for tile_server_options::TileServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileServerOptions").finish()
    }
}

impl Display for map_observer::MapLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::StyleParseError => "Failed parsing style",
            Self::StyleLoadError => "Failed loading style",
            Self::NotFoundError => "Style not found",
            Self::UnknownError => "Unknown error",
            _ => "Unrecognized error",
        };
        write!(f, "{s}")
    }
}

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
/// Map observer callbacks and related types.
pub mod map_observer {
    #[namespace = "mln::bridge"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Camera change mode for map observer callbacks.
    pub enum MapObserverCameraChangeMode {
        /// Camera changed immediately without animation.
        Immediate,
        /// Camera changed using an animated transition.
        Animated,
    }

    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Map loading error types.
    pub enum MapLoadError {
        /// Style parsing error.
        StyleParseError,
        /// Style loading error.
        StyleLoadError,
        /// Resource not found.
        NotFoundError,
        /// Unknown error.
        UnknownError,
    }

    #[namespace = "mbgl"]
    extern "C++" {
        include!("mbgl/map/map_observer.hpp");
        type MapLoadError;
    }

    // Declarations for C++ with implementations in Rust
    extern "Rust" {
        type VoidCallback;
        type FinishRenderingFrameCallback;
        type CameraDidChangeCallback;
        type FailingLoadingMapCallback;

        fn void_callback(callback: &VoidCallback);
        fn finish_rendering_frame_callback(
            callback: &FinishRenderingFrameCallback,
            needsRepaint: bool,
            placementChanged: bool,
        );
        fn camera_did_change_callback(
            callback: &CameraDidChangeCallback,
            mode: MapObserverCameraChangeMode,
        );
        fn failing_loading_map_callback(
            callback: &FailingLoadingMapCallback,
            error: MapLoadError,
            what: &str,
        );
    }

    // Declarations for Rust with implementations in C++
    extern "C++" {
        include!("map_observer.h"); // Required to find functions below

        type MapObserverCameraChangeMode;

        // C++ Opaque types
        #[rust_name = "CxxMapObserver"]
        type MapObserver = super::ffi::MapObserver; // Created custom map observer
    }

    unsafe extern "C++" {
        // With `self: Pin<&mut MapObserver>` as first argument, it is a non static method of that object.
        // cxx searches for such a method
        /// Sets the callback for when loading of the map will start.
        fn setWillStartLoadingMapCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        /// Sets the callback for when the style has finished loading.
        fn setFinishLoadingStyleCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        /// Sets the callback for when the map becomes idle.
        fn setBecomeIdleCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        /// Sets the callback for when loading of the map fails.
        fn setFailLoadingMapCallback(
            self: &CxxMapObserver,
            callback: Box<FailingLoadingMapCallback>,
        );
        /// Sets the callback for when a frame finishes rendering.
        fn setFinishRenderingFrameCallback(
            self: &CxxMapObserver,
            callback: Box<FinishRenderingFrameCallback>,
        );
        /// Sets the callback for when the camera finishes changing.
        fn setCameraDidChangeCallback(
            self: &CxxMapObserver,
            callback: Box<CameraDidChangeCallback>,
        );
    }
}

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
/// Core FFI definitions and types for the MapLibre bridge.
pub mod ffi {
    // CXX validates enum types against the C++ definition during compilation

    // The mbgl enums must be defined in the same namespace than on the C++ side
    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Map rendering mode configuration.
    pub enum MapMode {
        /// Continually updating map
        Continuous,
        /// Once-off still image of an arbitrary viewport
        Static,
        /// Once-off still image of a single tile
        Tile,
    }

    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Debug visualization options for map rendering.
    pub enum MapDebugOptions {
        /// No debug visualization.
        NoDebug = 0,
        /// Edges of tile boundaries are shown as thick, red lines.
        ///
        /// Can help diagnose tile clipping issues.
        TileBorders = 0b0000_0010, // 1 << 1
        /// Shows tile parsing status information.
        ParseStatus = 0b0000_0100, // 1 << 2
        /// Each tile shows a timestamp indicating when it was loaded.
        Timestamps = 0b0000_1000, // 1 << 3
        /// Edges of glyphs and symbols are shown as faint, green lines.
        ///
        /// Can help diagnose collision and label placement issues.
        Collision = 0b0001_0000, // 1 << 4
        /// Each drawing operation is replaced by a translucent fill.
        ///
        /// Overlapping drawing operations appear more prominent to help diagnose overdrawing.
        Overdraw = 0b0010_0000, // 1 << 5
        /// The stencil buffer is shown instead of the color buffer.
        ///
        /// Note: This option does nothing in Release builds of the SDK.
        StencilClip = 0b0100_0000, // 1 << 6
        /// The depth buffer is shown instead of the color buffer.
        ///
        /// Note: This option does nothing in Release builds of the SDK
        DepthBuffer = 0b1000_0000, // 1 << 7
    }

    /// MapLibre Native Event Severity levels
    #[namespace = "mbgl"]
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EventSeverity {
        /// Debug severity level.
        Debug = 0,
        /// Info severity level.
        Info = 1,
        /// Warning severity level.
        Warning = 2,
        /// Error severity level.
        Error = 3,
    }

    /// MapLibre Native Event types
    #[namespace = "mbgl"]
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Event {
        /// General event.
        General = 0,
        /// Setup event.
        Setup = 1,
        /// Shader event.
        Shader = 2,
        /// Style parsing event.
        ParseStyle = 3,
        /// Tile parsing event.
        ParseTile = 4,
        /// Render event.
        Render = 5,
        /// Style event.
        Style = 6,
        /// Database event.
        Database = 7,
        /// HTTP request event.
        HttpRequest = 8,
        /// Sprite event.
        Sprite = 9,
        /// Image event.
        Image = 10,
        /// OpenGL event.
        OpenGL = 11,
        /// JNI event.
        JNI = 12,
        /// Android event.
        Android = 13,
        /// Crash event.
        Crash = 14,
        /// Glyph event.
        Glyph = 15,
        /// Timing event.
        Timing = 16,
    }

    #[namespace = "mbgl"]
    extern "C++" {
        include!("mbgl/map/mode.hpp");

        type MapMode;
        type MapDebugOptions;
        // The name must be unique but for some reason this is required
        /// Resource configuration options.
        #[rust_name = "CxxResourceOptions"]
        type ResourceOptions = super::resource_options::ResourceOptions;
        /// Event severity enumeration.
        pub type EventSeverity;
        /// Event type enumeration.
        pub type Event;
    }

    #[namespace = "mbgl"]
    extern "C++" {
        /// Screen coordinate type.
        type ScreenCoordinate = super::ScreenCoordinate;
        /// Size type.
        type Size = super::Size;
    }

    #[namespace = "mbgl::style"]
    extern "C++" {
        /// GeoJSON source opaque type.
        #[rust_name = "CxxGeoJSONSource"]
        type GeoJSONSource = super::sources::GeoJSONSource;
        /// Symbol layer opaque type.
        #[rust_name = "CxxSymbolLayer"]
        type SymbolLayer = super::layers::SymbolLayer;
    }

    #[cfg(feature = "wgpu")]
    #[namespace = ""]
    #[repr(u32)]
    enum WGPUTextureDimension {
        WGPUTextureDimension_Undefined = 0x00000000,
        WGPUTextureDimension_1D = 0x00000001,
        WGPUTextureDimension_2D = 0x00000002,
        WGPUTextureDimension_3D = 0x00000003,
    }

    #[cfg(feature = "wgpu")]
    #[namespace = ""]
    #[repr(u32)]
    enum WGPUTextureAspect {
        WGPUTextureAspect_Undefined = 0x00000000,
        WGPUTextureAspect_All = 0x00000001,
        WGPUTextureAspect_StencilOnly = 0x00000002,
        WGPUTextureAspect_DepthOnly = 0x00000003,
    }

    #[cfg(feature = "wgpu")]
    #[namespace = ""]
    #[repr(u32)]
    enum WGPUTextureViewDimension {
        WGPUTextureViewDimension_Undefined = 0x00000000,
        WGPUTextureViewDimension_1D = 0x00000001,
        WGPUTextureViewDimension_2D = 0x00000002,
        WGPUTextureViewDimension_2DArray = 0x00000003,
        WGPUTextureViewDimension_Cube = 0x00000004,
        WGPUTextureViewDimension_CubeArray = 0x00000005,
        WGPUTextureViewDimension_3D = 0x00000006,
    }

    #[cfg(feature = "wgpu")]
    #[namespace = ""]
    #[repr(u32)]
    enum WGPUTextureFormat {
        WGPUTextureFormat_Undefined = 0x00000000,
        WGPUTextureFormat_R8Unorm = 0x00000001,
        WGPUTextureFormat_R8Snorm = 0x00000002,
        WGPUTextureFormat_R8Uint = 0x00000003,
        WGPUTextureFormat_R8Sint = 0x00000004,
        WGPUTextureFormat_R16Unorm = 0x00000005,
        WGPUTextureFormat_R16Snorm = 0x00000006,
        WGPUTextureFormat_R16Uint = 0x00000007,
        WGPUTextureFormat_R16Sint = 0x00000008,
        WGPUTextureFormat_R16Float = 0x00000009,
        WGPUTextureFormat_RG8Unorm = 0x0000000A,
        WGPUTextureFormat_RG8Snorm = 0x0000000B,
        WGPUTextureFormat_RG8Uint = 0x0000000C,
        WGPUTextureFormat_RG8Sint = 0x0000000D,
        WGPUTextureFormat_R32Float = 0x0000000E,
        WGPUTextureFormat_R32Uint = 0x0000000F,
        WGPUTextureFormat_R32Sint = 0x00000010,
        WGPUTextureFormat_RG16Unorm = 0x00000011,
        WGPUTextureFormat_RG16Snorm = 0x00000012,
        WGPUTextureFormat_RG16Uint = 0x00000013,
        WGPUTextureFormat_RG16Sint = 0x00000014,
        WGPUTextureFormat_RG16Float = 0x00000015,
        WGPUTextureFormat_RGBA8Unorm = 0x00000016,
        WGPUTextureFormat_RGBA8UnormSrgb = 0x00000017,
        WGPUTextureFormat_RGBA8Snorm = 0x00000018,
        WGPUTextureFormat_RGBA8Uint = 0x00000019,
        WGPUTextureFormat_RGBA8Sint = 0x0000001A,
        WGPUTextureFormat_BGRA8Unorm = 0x0000001B,
        WGPUTextureFormat_BGRA8UnormSrgb = 0x0000001C,
        WGPUTextureFormat_RGB10A2Uint = 0x0000001D,
        WGPUTextureFormat_RGB10A2Unorm = 0x0000001E,
        WGPUTextureFormat_RG11B10Ufloat = 0x0000001F,
        WGPUTextureFormat_RGB9E5Ufloat = 0x00000020,
        WGPUTextureFormat_RG32Float = 0x00000021,
        WGPUTextureFormat_RG32Uint = 0x00000022,
        WGPUTextureFormat_RG32Sint = 0x00000023,
        WGPUTextureFormat_RGBA16Unorm = 0x00000024,
        WGPUTextureFormat_RGBA16Snorm = 0x00000025,
        WGPUTextureFormat_RGBA16Uint = 0x00000026,
        WGPUTextureFormat_RGBA16Sint = 0x00000027,
        WGPUTextureFormat_RGBA16Float = 0x00000028,
        WGPUTextureFormat_RGBA32Float = 0x00000029,
        WGPUTextureFormat_RGBA32Uint = 0x0000002A,
        WGPUTextureFormat_RGBA32Sint = 0x0000002B,
        WGPUTextureFormat_Stencil8 = 0x0000002C,
        WGPUTextureFormat_Depth16Unorm = 0x0000002D,
        WGPUTextureFormat_Depth24Plus = 0x0000002E,
        WGPUTextureFormat_Depth24PlusStencil8 = 0x0000002F,
        WGPUTextureFormat_Depth32Float = 0x00000030,
        WGPUTextureFormat_Depth32FloatStencil8 = 0x00000031,
        WGPUTextureFormat_BC1RGBAUnorm = 0x00000032,
        WGPUTextureFormat_BC1RGBAUnormSrgb = 0x00000033,
        WGPUTextureFormat_BC2RGBAUnorm = 0x00000034,
        WGPUTextureFormat_BC2RGBAUnormSrgb = 0x00000035,
        WGPUTextureFormat_BC3RGBAUnorm = 0x00000036,
        WGPUTextureFormat_BC3RGBAUnormSrgb = 0x00000037,
        WGPUTextureFormat_BC4RUnorm = 0x00000038,
        WGPUTextureFormat_BC4RSnorm = 0x00000039,
        WGPUTextureFormat_BC5RGUnorm = 0x0000003A,
        WGPUTextureFormat_BC5RGSnorm = 0x0000003B,
        WGPUTextureFormat_BC6HRGBUfloat = 0x0000003C,
        WGPUTextureFormat_BC6HRGBFloat = 0x0000003D,
        WGPUTextureFormat_BC7RGBAUnorm = 0x0000003E,
        WGPUTextureFormat_BC7RGBAUnormSrgb = 0x0000003F,
        WGPUTextureFormat_ETC2RGB8Unorm = 0x00000040,
        WGPUTextureFormat_ETC2RGB8UnormSrgb = 0x00000041,
        WGPUTextureFormat_ETC2RGB8A1Unorm = 0x00000042,
        WGPUTextureFormat_ETC2RGB8A1UnormSrgb = 0x00000043,
        WGPUTextureFormat_ETC2RGBA8Unorm = 0x00000044,
        WGPUTextureFormat_ETC2RGBA8UnormSrgb = 0x00000045,
        WGPUTextureFormat_EACR11Unorm = 0x00000046,
        WGPUTextureFormat_EACR11Snorm = 0x00000047,
        WGPUTextureFormat_EACRG11Unorm = 0x00000048,
        WGPUTextureFormat_EACRG11Snorm = 0x00000049,
        WGPUTextureFormat_ASTC4x4Unorm = 0x0000004A,
        WGPUTextureFormat_ASTC4x4UnormSrgb = 0x0000004B,
        WGPUTextureFormat_ASTC5x4Unorm = 0x0000004C,
        WGPUTextureFormat_ASTC5x4UnormSrgb = 0x0000004D,
        WGPUTextureFormat_ASTC5x5Unorm = 0x0000004E,
        WGPUTextureFormat_ASTC5x5UnormSrgb = 0x0000004F,
        WGPUTextureFormat_ASTC6x5Unorm = 0x00000050,
        WGPUTextureFormat_ASTC6x5UnormSrgb = 0x00000051,
        WGPUTextureFormat_ASTC6x6Unorm = 0x00000052,
        WGPUTextureFormat_ASTC6x6UnormSrgb = 0x00000053,
        WGPUTextureFormat_ASTC8x5Unorm = 0x00000054,
        WGPUTextureFormat_ASTC8x5UnormSrgb = 0x00000055,
        WGPUTextureFormat_ASTC8x6Unorm = 0x00000056,
        WGPUTextureFormat_ASTC8x6UnormSrgb = 0x00000057,
        WGPUTextureFormat_ASTC8x8Unorm = 0x00000058,
        WGPUTextureFormat_ASTC8x8UnormSrgb = 0x00000059,
        WGPUTextureFormat_ASTC10x5Unorm = 0x0000005A,
        WGPUTextureFormat_ASTC10x5UnormSrgb = 0x0000005B,
        WGPUTextureFormat_ASTC10x6Unorm = 0x0000005C,
        WGPUTextureFormat_ASTC10x6UnormSrgb = 0x0000005D,
        WGPUTextureFormat_ASTC10x8Unorm = 0x0000005E,
        WGPUTextureFormat_ASTC10x8UnormSrgb = 0x0000005F,
        WGPUTextureFormat_ASTC10x10Unorm = 0x00000060,
        WGPUTextureFormat_ASTC10x10UnormSrgb = 0x00000061,
        WGPUTextureFormat_ASTC12x10Unorm = 0x00000062,
        WGPUTextureFormat_ASTC12x10UnormSrgb = 0x00000063,
        WGPUTextureFormat_ASTC12x12Unorm = 0x00000064,
        WGPUTextureFormat_ASTC12x12UnormSrgb = 0x00000065,
    }

    #[cfg(feature = "wgpu")]
    #[namespace = ""]
    extern "C++" {
        // include!("webgpu/webgpu.h");

        type WGPUTextureDimension;
        type WGPUTextureFormat;
        type WGPUTextureUsage = super::WGPUTextureUsage;
        type WGPUExtent3D = super::WGPUExtent3D;
        type WGPUTextureViewDimension;
        type WGPUTextureAspect;
    }

    // Declarations for Rust with implementations in C++
    unsafe extern "C++" {
        include!("map_renderer.h");

        // C++ Opaque types
        /// Bridge image for rendering output.
        type BridgeImage;
        /// Map observer for handling map events.
        type MapObserver; // Created custom map observer
        /// Map renderer for rendering map content.
        type MapRenderer;
        type Texture;
        type TextureView;
        /// Creates a new map renderer instance.
        #[allow(clippy::too_many_arguments)]
        fn MapRenderer_new(
            mapMode: MapMode,
            width: u32,
            height: u32,
            pixelRatio: f32,
            resource_options: &CxxResourceOptions,
        ) -> UniquePtr<MapRenderer>;
        /// Reads the current still image from the renderer.
        fn readStillImage(self: Pin<&mut MapRenderer>) -> UniquePtr<BridgeImage>;
        /// Gets the pixel data pointer from a bridge image.
        fn get(self: &BridgeImage) -> *const u8;
        /// Gets the size of a bridge image.
        fn size(self: &BridgeImage) -> Size;
        /// Gets the buffer length of a bridge image.
        fn bufferLength(self: &BridgeImage) -> usize;
        /// Renders a single frame.
        fn render_once(self: Pin<&mut MapRenderer>);
        /// Renders continuously.
        fn render(self: Pin<&mut MapRenderer>) -> UniquePtr<CxxString>;
        /// Sets debug visualization flags.
        fn setDebugFlags(self: Pin<&mut MapRenderer>, flags: MapDebugOptions);
        /// Sets the camera position and orientation.
        fn setCamera(
            self: Pin<&mut MapRenderer>,
            lat: f64,
            lon: f64,
            zoom: f64,
            bearing: f64,
            pitch: f64,
        );
        /// Moves the camera by the given delta.
        fn moveBy(self: Pin<&mut MapRenderer>, delta: &ScreenCoordinate);
        /// Scales the camera based on the given scale factor.
        fn scaleBy(self: Pin<&mut MapRenderer>, scale: f64, pos: &ScreenCoordinate);
        /// Loads a style from a URL.
        fn style_load_from_url(self: Pin<&mut MapRenderer>, url: &str);
        /// Sets the renderer size.
        fn setSize(self: Pin<&mut MapRenderer>, size: &Size);
        /// Gets the map observer.
        fn observer(self: Pin<&mut MapRenderer>) -> SharedPtr<MapObserver>;
        /// Adds an image to the style.
        fn style_add_image(
            self: Pin<&mut MapRenderer>,
            id: &str,
            data: &[u8],
            size: Size,
            single_distance_field: bool,
        );
        /// Removes an image from the style.
        fn style_remove_image(self: Pin<&mut MapRenderer>, id: &str);
        /// Adds a GeoJSON source to the style.
        fn style_add_geojson_source(
            self: Pin<&mut MapRenderer>,
            source: UniquePtr<CxxGeoJSONSource>,
        );
        /// Adds a symbol layer to the style.
        fn style_add_symbol_layer(self: Pin<&mut MapRenderer>, layer: UniquePtr<CxxSymbolLayer>);

        // Texture
        fn getTexture(self: Pin<&mut MapRenderer>) -> UniquePtr<Texture>;
        #[cfg(feature = "wgpu")]
        fn createView(
            self: &Texture,
            format: WGPUTextureFormat,
            dimension: WGPUTextureViewDimension,
            usage: WGPUTextureUsage,
            aspect: WGPUTextureAspect,
            base_mip_level: u32,
            mip_level_count: u32,
            base_array_layer: u32,
            array_layer_count: u32,
        ) -> UniquePtr<TextureView>;
        #[cfg(feature = "wgpu")]
        fn destroy(self: &Texture);
        #[cfg(feature = "wgpu")]
        fn getMipLevelCount(self: &Texture) -> u32;
        #[cfg(feature = "wgpu")]
        fn getSampleCount(self: &Texture) -> u32;
        #[cfg(feature = "wgpu")]
        fn getDimension(self: &Texture) -> WGPUTextureDimension;
        #[cfg(feature = "wgpu")]
        fn getFormat(self: &Texture) -> WGPUTextureFormat;
        #[cfg(feature = "wgpu")]
        fn getUsage(self: &Texture) -> WGPUTextureUsage;
        #[cfg(feature = "wgpu")]
        fn getExtend3d(self: &Texture) -> WGPUExtent3D;
    }

    // Declarations for C++ with implementations in Rust
    extern "Rust" {
        /// Bridge logging from C++ to Rust log crate
        fn log_from_cpp(severity: EventSeverity, event: Event, code: i64, message: &str);
    }

    unsafe extern "C++" {
        include!("rust_log_observer.h");

        /// Enables or disables logging from a separate thread.
        fn Log_useLogThread(enable: bool);
    }
}

impl std::fmt::Debug for ffi::BridgeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgeImage").finish()
    }
}

impl std::fmt::Debug for ffi::MapObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapObserver").finish()
    }
}

impl std::fmt::Debug for ffi::MapRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapRenderer").finish()
    }
}

unsafe impl cxx::ExternType for Size {
    type Id = cxx::type_id!("mbgl::Size");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for ScreenCoordinate {
    type Id = cxx::type_id!("mbgl::ScreenCoordinate");
    type Kind = cxx::kind::Trivial;
}

#[cfg(feature = "wgpu")]
pub mod wgpu {
    use cxx::UniquePtr;
    pub struct TextureInterface(pub UniquePtr<super::ffi::Texture>);
    pub struct TextureViewInterface(pub UniquePtr<super::ffi::TextureView>);

    impl std::fmt::Debug for TextureInterface {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TextureInterface")
        }
    }

    impl wgpu::custom::TextureInterface for TextureInterface {
        fn create_view(
            &self,
            desc: &wgpu::TextureViewDescriptor<'_>,
        ) -> wgpu::custom::DispatchTextureView {
            // TODO: get rid of unwraps!
            let format = if let Some(format) = desc.format {
                format.try_into().unwrap()
            } else {
                self.0.getFormat()
            };
            let dimension = desc.dimension.unwrap().try_into().unwrap(); // _or(self.0.getDimension().0);
            let usage = if let Some(usage) = desc.usage {
                usage.try_into().unwrap()
            } else {
                self.0.getUsage()
            };

            let aspect = desc.aspect.try_into().unwrap();
            let base_mip_level = desc.base_mip_level;
            let mip_level_count = desc.mip_level_count.unwrap();
            let base_array_layer = desc.base_array_layer;
            let array_layer_count = desc.array_layer_count.unwrap(); // _or(default)
            wgpu::custom::DispatchTextureView::custom(TextureViewInterface(self.0.createView(
                format,
                dimension,
                usage,
                aspect,
                base_mip_level,
                mip_level_count,
                base_array_layer,
                array_layer_count,
            )))
        }

        fn destroy(&self) {
            self.0.destroy();
        }
    }

    unsafe impl Send for TextureInterface {}
    unsafe impl Sync for TextureInterface {}

    impl std::fmt::Debug for TextureViewInterface {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TextureViewInterface")
        }
    }

    impl wgpu::custom::TextureViewInterface for TextureViewInterface {}
    unsafe impl Send for TextureViewInterface {}
    unsafe impl Sync for TextureViewInterface {}
}

#[cfg(test)]
mod test {
    use crate::{ScreenCoordinate, X, Y};

    #[test]
    fn screen_corrdinate_diff() {
        let s1 = ScreenCoordinate::new(X(5.), Y(-1.));
        let s2 = ScreenCoordinate::new(X(3.), Y(-10.));

        let res = s1 - s2;
        assert!((res.x - 2.).abs() < 0.00001);
        assert!((res.y - 9.).abs() < 0.00001);
    }
}
