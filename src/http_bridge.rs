#[cfg(feature = "wgpu")]
use std::ffi::{CString, c_char};
#[cfg(feature = "wgpu")]
use std::io::Read;

#[cfg(feature = "wgpu")]
use cxx::CxxString;

#[cfg(feature = "wgpu")]
unsafe extern "C" {
    fn mbgl_rust_http_set_bridge(
        request_fn: Option<
            unsafe extern "C" fn(*const CxxString, u8, *mut HttpResponse) -> bool,
        >,
    );
}

#[cxx::bridge]
mod ffi {
    enum Reason {
        Success,
        NotFound,
        Server,
        Connection,
        RateLimit,
        Other,
    }

    extern "C++" {
        include!("response.h");
        type Reason;
    }
}

#[cfg(feature = "wgpu")]
unsafe extern "C" fn rust_http_bridge_request(
    url: *const CxxString,
    _kind: u8,
    out_response: *mut HttpResponse,
) -> bool {
    let set_error = |response: &mut RustHttpResponseBridge, reason: u8, message: String| {
        let sanitized = message.replace('\0', " ");
        if let Ok(msg) = CString::new(sanitized) {
            response.setError(reason,  = msg.into_raw())
        }
    };

    if out_response.is_null() || url.is_null() {
        return false;
    }

    let url_str = match unsafe { (&*url).to_str() } {
        Ok(value) => value,
        Err(err) => {
            unsafe {
                set_error(
                    &mut *out_response,
                    Reason::Other,
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
                    (*out_response).setNoContent(true);
                    return true;
                }
                if status == 304 {
                    (*out_response).setNotModified(true);
                    return true;
                }
                if let Some(etag) = response.header("ETag") {
                        (*out_response).setETag(etag.as_bytes());
                }
            }

            let mut bytes = Vec::new();
            let mut reader = response.into_reader();
            if let Err(err) = reader.read_to_end(&mut bytes) {
                unsafe {
                    set_error(
                        &mut *out_response,
                        Reason::Connection,
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
                Reason::NotFound
            } else if status == 429 {
                Reason::RateLimit
            } else if status >= 500 {
                Reason::Server
            } else {
                Reason::Other
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
                    Reason::Connection,
                    format!("HTTP transport error: {err}"),
                );
            }
            true
        }
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
