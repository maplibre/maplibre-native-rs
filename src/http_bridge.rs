#[cfg(feature = "wgpu")]
use std::io::Read;
#[cfg(feature = "wgpu")]
use std::pin::Pin;

#[cxx::bridge]
mod ffi {
    #[namespace = "mln::bridge"]
    enum Reason {
        Success = 1,
        NotFound,
        Server,
        Connection,
        RateLimit,
        Other,
    }

    #[namespace = "mln::bridge"]
    enum Kind {
        Unknown,
        Style,
        Source,
        Tile,
        Glyphs,
        SpriteImage,
        SpriteJSON,
        Image,
    }

    #[namespace = "mln::bridge"]
    unsafe extern "C++" {
        include!("response.h");

        type Reason;
        type Kind;

        #[namespace = "mbgl"]
        type HttpResponse;

        fn http_response_set_data(response: Pin<&mut HttpResponse>, data: &[u8]);
        fn http_response_set_etag(response: Pin<&mut HttpResponse>, etag: &[u8]);
        fn http_response_set_no_content(response: Pin<&mut HttpResponse>, no_content: bool);
        fn http_response_set_not_modified(response: Pin<&mut HttpResponse>, not_modified: bool);
        fn http_response_set_error(
            response: Pin<&mut HttpResponse>,
            reason: Reason,
            error_message: &str,
        );
    }
}

#[cfg(feature = "wgpu")]
use ffi::{HttpResponse, Kind, Reason};

#[cfg(feature = "wgpu")]
unsafe extern "C" {
    fn mbgl_rust_http_set_bridge(
        request_fn: Option<
            unsafe extern "C" fn(*const cxx::CxxString, Kind, *mut HttpResponse) -> bool,
        >,
    );
}

#[cfg(feature = "wgpu")]
fn set_error(response: Pin<&mut HttpResponse>, reason: Reason, message: impl AsRef<str>) {
    ffi::http_response_set_error(response, reason, message.as_ref().replace('\0', " ").as_str());
}

#[cfg(feature = "wgpu")]
unsafe extern "C" fn rust_http_bridge_request(
    url: *const cxx::CxxString,
    _kind: ffi::Kind,
    out_response: *mut HttpResponse,
) -> bool {
    if out_response.is_null() || url.is_null() {
        return false;
    }

    let mut out_response = unsafe { Pin::new_unchecked(&mut *out_response) };

    let url_str = match unsafe { (&*url).to_str() } {
        Ok(value) => value,
        Err(err) => {
            set_error(
                out_response.as_mut(),
                Reason::Other,
                format!("Invalid URL string in Rust HTTP bridge: {err}"),
            );
            return true;
        }
    };

    #[cfg(feature = "ureq_http")]
    {
        let request = ureq::get(url_str).set("User-Agent", "maplibre-native-rs-http-bridge");
        match request.call() {
            Ok(response) => {
                let status = response.status();
                if status == 204 {
                    ffi::http_response_set_no_content(out_response.as_mut(), true);
                    return true;
                }
                if status == 304 {
                    ffi::http_response_set_not_modified(out_response.as_mut(), true);
                    return true;
                }
                if let Some(etag) = response.header("ETag") {
                    ffi::http_response_set_etag(out_response.as_mut(), etag.as_bytes());
                }

                let mut bytes = Vec::new();
                let mut reader = response.into_reader();
                if let Err(err) = reader.read_to_end(&mut bytes) {
                    set_error(
                        out_response.as_mut(),
                        Reason::Connection,
                        format!("Failed reading HTTP response body: {err}"),
                    );
                    return true;
                }

                if !bytes.is_empty() {
                    ffi::http_response_set_data(out_response.as_mut(), &bytes);
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

                if status == 404 {
                    ffi::http_response_set_no_content(out_response.as_mut(), true);
                }

                let status_text = response.status_text();
                let message = if status_text.is_empty() {
                    format!("HTTP status {status}")
                } else {
                    format!("HTTP status {status}: {status_text}")
                };
                set_error(out_response.as_mut(), reason, message);

                true
            }
            Err(ureq::Error::Transport(err)) => {
                set_error(
                    out_response.as_mut(),
                    Reason::Connection,
                    format!("HTTP transport error: {err}"),
                );
                true
            }
        }
    }
}

#[cfg(feature = "wgpu")]
/// Registers the default Rust HTTP bridge callbacks used by MapLibre's Rust platform HTTP file source.
pub fn init_rust_http_bridge() {
    unsafe {
        mbgl_rust_http_set_bridge(Some(rust_http_bridge_request));
    }
}

#[cfg(not(feature = "wgpu"))]
/// No-op when the `wgpu` feature is disabled.
pub fn init_rust_http_bridge() {}
