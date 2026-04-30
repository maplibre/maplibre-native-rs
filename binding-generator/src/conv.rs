use wgpu::{AddressMode, CompareFunction, FilterMode, MipmapFilterMode, SamplerDescriptor};

use crate::{
    WGPUAddressMode_MirrorRepeat, WGPUAddressMode_Repeat, WGPUCompareFunction_Always,
    WGPUCompareFunction_Equal, WGPUCompareFunction_Greater, WGPUCompareFunction_GreaterEqual,
    WGPUCompareFunction_Less, WGPUCompareFunction_LessEqual, WGPUCompareFunction_Never,
    WGPUCompareFunction_NotEqual, WGPUFilterMode_Linear, WGPUMipmapFilterMode_Linear,
    WGPUSamplerDescriptor, WGPUStringView,
};

/// Convert a `WGPUStringView` to an `Option<&str>`.
/// Returns `None` for the null sentinel `{NULL, WGPU_STRLEN}`.
///
/// # Safety
/// The caller must ensure that `view.data` points to valid UTF-8 memory for
/// `view.length` bytes (or is null-terminated when `length == WGPU_STRLEN`).
unsafe fn string_view<'a>(view: WGPUStringView) -> Option<&'a str> {
    if view.data.is_null() {
        return None;
    }
    let bytes = if view.length == usize::MAX {
        // WGPU_STRLEN — treat as null-terminated
        unsafe { std::ffi::CStr::from_ptr(view.data) }.to_bytes()
    } else {
        unsafe { std::slice::from_raw_parts(view.data as *const u8, view.length) }
    };
    std::str::from_utf8(bytes).ok()
}

pub fn sampler_descriptor<'a>(d: &'a WGPUSamplerDescriptor) -> SamplerDescriptor<'a> {
    SamplerDescriptor {
        label: unsafe { string_view(d.label) },
        address_mode_u: map_address_mode(d.addressModeU),
        address_mode_v: map_address_mode(d.addressModeV),
        address_mode_w: map_address_mode(d.addressModeW),
        mag_filter: map_filter_mode(d.magFilter),
        min_filter: map_filter_mode(d.minFilter),
        mipmap_filter: map_mipmap_filter_mode(d.mipmapFilter),
        lod_min_clamp: d.lodMinClamp,
        lod_max_clamp: d.lodMaxClamp,
        compare: map_compare_function(d.compare),
        anisotropy_clamp: d.maxAnisotropy,
        border_color: None,
    }
}

fn map_address_mode(mode: crate::WGPUAddressMode) -> AddressMode {
    match mode {
        WGPUAddressMode_Repeat => AddressMode::Repeat,
        WGPUAddressMode_MirrorRepeat => AddressMode::MirrorRepeat,
        _ => AddressMode::ClampToEdge,
    }
}

fn map_filter_mode(mode: crate::WGPUFilterMode) -> FilterMode {
    match mode {
        WGPUFilterMode_Linear => FilterMode::Linear,
        _ => FilterMode::Nearest,
    }
}

fn map_mipmap_filter_mode(mode: crate::WGPUMipmapFilterMode) -> MipmapFilterMode {
    match mode {
        WGPUMipmapFilterMode_Linear => MipmapFilterMode::Linear,
        _ => MipmapFilterMode::Nearest,
    }
}

fn map_compare_function(func: crate::WGPUCompareFunction) -> Option<CompareFunction> {
    match func {
        WGPUCompareFunction_Never => Some(CompareFunction::Never),
        WGPUCompareFunction_Less => Some(CompareFunction::Less),
        WGPUCompareFunction_Equal => Some(CompareFunction::Equal),
        WGPUCompareFunction_LessEqual => Some(CompareFunction::LessEqual),
        WGPUCompareFunction_Greater => Some(CompareFunction::Greater),
        WGPUCompareFunction_NotEqual => Some(CompareFunction::NotEqual),
        WGPUCompareFunction_GreaterEqual => Some(CompareFunction::GreaterEqual),
        WGPUCompareFunction_Always => Some(CompareFunction::Always),
        _ => None,
    }
}
