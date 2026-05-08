#[cfg(feature = "wgpu")]
use std::ffi::{CString, c_char};
#[cfg(feature = "wgpu")]
use std::io::Read;

#[cfg(feature = "wgpu")]
use cxx::CxxString;

#[cfg(feature = "wgpu")]
#[repr(C)]
#[derive(Default)]
struct RustHttpResponseBridge {
    data: *const u8,
    data_len: usize,
    etag: *const c_char,
    error_message: *const c_char,
    error_reason: u8,
    no_content: bool,
    not_modified: bool,
}

#[cfg(feature = "wgpu")]
unsafe extern "C" {
    fn mbgl_rust_http_set_bridge(
        request_fn: Option<
            unsafe extern "C" fn(*const CxxString, u8, *mut RustHttpResponseBridge) -> bool,
        >,
        release_fn: Option<unsafe extern "C" fn(*mut RustHttpResponseBridge)>,
    );
}

#[cfg(feature = "wgpu")]
unsafe extern "C" fn rust_http_bridge_request(
    url: *const CxxString,
    _kind: u8,
    out_response: *mut RustHttpResponseBridge,
) -> bool {
    const REASON_NOT_FOUND: u8 = 2;
    const REASON_SERVER: u8 = 3;
    const REASON_CONNECTION: u8 = 4;
    const REASON_RATE_LIMIT: u8 = 5;
    const REASON_OTHER: u8 = 6;

    let set_error = |response: &mut RustHttpResponseBridge, reason: u8, message: String| {
        let sanitized = message.replace('\0', " ");
        if let Ok(msg) = CString::new(sanitized) {
            response.error_reason = reason;
            response.error_message = msg.into_raw();
        }
    };

    if out_response.is_null() || url.is_null() {
        return false;
    }

    unsafe {
        *out_response = RustHttpResponseBridge::default();
    }

    let url_str = match unsafe { (&*url).to_str() } {
        Ok(value) => value,
        Err(err) => {
            unsafe {
                set_error(
                    &mut *out_response,
                    REASON_OTHER,
                    format!("Invalid URL string in Rust HTTP bridge: {err}"),
                );
            }
            return true;
        }
    };

    let request = ureq::get(url_str).set("User-Agent", "maplibre-native-rs-http-bridge");
    match request.call() {
        Ok(response) => {
            let status = response.status();
            unsafe {
                if status == 204 {
                    (*out_response).no_content = true;
                    return true;
                }
                if status == 304 {
                    (*out_response).not_modified = true;
                    return true;
                }
                if let Some(etag) = response.header("ETag") {
                    let sanitized = etag.replace('\0', " ");
                    if let Ok(etag_c) = CString::new(sanitized) {
                        (*out_response).etag = etag_c.into_raw();
                    }
                }
            }

            let mut bytes = Vec::new();
            let mut reader = response.into_reader();
            if let Err(err) = reader.read_to_end(&mut bytes) {
                unsafe {
                    set_error(
                        &mut *out_response,
                        REASON_CONNECTION,
                        format!("Failed reading HTTP response body: {err}"),
                    );
                }
                return true;
            }

            if !bytes.is_empty() {
                let len = bytes.len();
                let data = bytes.into_boxed_slice();
                let leaked: &'static mut [u8] = Box::leak(data);
                unsafe {
                    (*out_response).data = leaked.as_ptr();
                    (*out_response).data_len = len;
                }
            }

            true
        }
        Err(ureq::Error::Status(status, response)) => {
            let reason = if status == 404 {
                REASON_NOT_FOUND
            } else if status == 429 {
                REASON_RATE_LIMIT
            } else if status >= 500 {
                REASON_SERVER
            } else {
                REASON_OTHER
            };

            let message = {
                let status_text = response.status_text();
                if status_text.is_empty() {
                    format!("HTTP status {status}")
                } else {
                    format!("HTTP status {status}: {status_text}")
                }
            };

            unsafe {
                if status == 404 {
                    (*out_response).no_content = true;
                }
                set_error(&mut *out_response, reason, message);
            }

            true
        }
        Err(ureq::Error::Transport(err)) => {
            unsafe {
                set_error(
                    &mut *out_response,
                    REASON_CONNECTION,
                    format!("HTTP transport error: {err}"),
                );
            }
            true
        }
    }
}

#[cfg(feature = "wgpu")]
unsafe fn free_owned_data(response: &mut RustHttpResponseBridge) {
    if !response.data.is_null() && response.data_len > 0 {
        // Rebuild the boxed slice that was leaked in `rust_http_bridge_request`.
        let raw = core::ptr::slice_from_raw_parts_mut(response.data.cast_mut(), response.data_len);
        let _ = Box::from_raw(raw);
    }
}

#[cfg(feature = "wgpu")]
unsafe fn free_owned_c_string(ptr: *const c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr.cast_mut());
    }
}

#[cfg(feature = "wgpu")]
unsafe extern "C" fn rust_http_bridge_release(response: *mut RustHttpResponseBridge) {
    if response.is_null() {
        return;
    }

    unsafe {
        free_owned_c_string((*response).etag);
        free_owned_c_string((*response).error_message);
        free_owned_data(&mut *response);
        *response = RustHttpResponseBridge::default();
    }
}

#[cfg(feature = "wgpu")]
/// Registers the default Rust HTTP bridge callbacks used by MapLibre's Rust platform HTTP file source.
pub fn init_rust_http_bridge() {
    unsafe {
        mbgl_rust_http_set_bridge(Some(rust_http_bridge_request), Some(rust_http_bridge_release));
    }
}

#[cfg(not(feature = "wgpu"))]
/// No-op when the `wgpu` feature is disabled.
pub fn init_rust_http_bridge() {}
