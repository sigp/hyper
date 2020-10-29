use bytes::Bytes;
use libc::{c_int, size_t};
use std::ffi::c_void;

use super::body::hyper_body;
use super::task::{hyper_task_return_type, AsTaskType};
use super::{hyper_code, HYPER_ITER_CONTINUE};
use crate::header::{HeaderName, HeaderValue};
use crate::{Body, HeaderMap, Method, Request, Response, Uri};

pub struct hyper_request(pub(super) Request<Body>);

pub struct hyper_response(pub(super) Response<Body>);

pub struct hyper_headers {
    pub(super) headers: HeaderMap,
    orig_casing: HeaderCaseMap,
}

// Will probably be moved to `hyper::ext::http1`
#[derive(Debug, Default)]
pub(crate) struct HeaderCaseMap(HeaderMap<Bytes>);

// ===== impl hyper_request =====

ffi_fn! {
    /// Construct a new HTTP request.
    fn hyper_request_new() -> *mut hyper_request {
        Box::into_raw(Box::new(hyper_request(Request::new(Body::empty()))))
    }
}

ffi_fn! {
    /// Free an HTTP request if not going to send it on a client.
    fn hyper_request_free(req: *mut hyper_request) {
        drop(unsafe { Box::from_raw(req) });
    }
}

ffi_fn! {
    /// Set the HTTP Method of the request.
    fn hyper_request_set_method(req: *mut hyper_request, method: *const u8, method_len: size_t) -> hyper_code {
        let bytes = unsafe {
            std::slice::from_raw_parts(method, method_len as usize)
        };
        match Method::from_bytes(bytes) {
            Ok(m) => {
                *unsafe { &mut *req }.0.method_mut() = m;
                hyper_code::HYPERE_OK
            },
            Err(_) => {
                hyper_code::HYPERE_INVALID_ARG
            }
        }
    }
}

ffi_fn! {
    /// Set the URI of the request.
    fn hyper_request_set_uri(req: *mut hyper_request, uri: *const u8, uri_len: size_t) -> hyper_code {
        let bytes = unsafe {
            std::slice::from_raw_parts(uri, uri_len as usize)
        };
        match Uri::from_maybe_shared(bytes) {
            Ok(u) => {
                *unsafe { &mut *req }.0.uri_mut() = u;
                hyper_code::HYPERE_OK
            },
            Err(_) => {
                hyper_code::HYPERE_INVALID_ARG
            }
        }
    }
}

ffi_fn! {
    /// Gets a reference to the HTTP headers of this request
    ///
    /// This is not an owned reference, so it should not be accessed after the
    /// `hyper_request` has been consumed.
    fn hyper_request_headers(req: *mut hyper_request) -> *mut hyper_headers {
        hyper_headers::get_or_default(unsafe { &mut *req }.0.extensions_mut())
    }
}

ffi_fn! {
    /// Set the body of the request.
    ///
    /// The default is an empty body.
    ///
    /// This takes ownership of the `hyper_body *`, you must not use it or
    /// free it after setting it on the request.
    fn hyper_request_set_body(req: *mut hyper_request, body: *mut hyper_body) -> hyper_code {
        let body = unsafe { Box::from_raw(body) };
        *unsafe { &mut *req }.0.body_mut() = body.0;
        hyper_code::HYPERE_OK
    }
}

impl hyper_request {
    pub(super) fn finalize_request(&mut self) {
        if let Some(headers) = self.0.extensions_mut().remove::<hyper_headers>() {
            *self.0.headers_mut() = headers.headers;
            self.0.extensions_mut().insert(headers.orig_casing);
        }
    }
}

// ===== impl hyper_response =====

ffi_fn! {
    /// Free an HTTP response after using it.
    fn hyper_response_free(resp: *mut hyper_response) {
        drop(unsafe { Box::from_raw(resp) });
    }
}

ffi_fn! {
    /// Get the HTTP-Status code of this response.
    ///
    /// It will always be within the range of 100-599.
    fn hyper_response_status(resp: *const hyper_response) -> u16 {
        unsafe { &*resp }.0.status().as_u16()
    }
}

ffi_fn! {
    /// Gets a reference to the HTTP headers of this response.
    ///
    /// This is not an owned reference, so it should not be accessed after the
    /// `hyper_response` has been freed.
    fn hyper_response_headers(resp: *mut hyper_response) -> *mut hyper_headers {
        hyper_headers::get_or_default(unsafe { &mut *resp }.0.extensions_mut())
    }
}

ffi_fn! {
    /// Take ownership of the body of this response.
    ///
    /// It is safe to free the response even after taking ownership of its body.
    fn hyper_response_body(resp: *mut hyper_response) -> *mut hyper_body {
        let body = std::mem::take(unsafe { &mut *resp }.0.body_mut());
        Box::into_raw(Box::new(hyper_body(body)))
    }
}

impl hyper_response {
    pub(super) fn wrap(mut resp: Response<Body>) -> hyper_response {
        let headers = std::mem::take(resp.headers_mut());
        let orig_casing = resp
            .extensions_mut()
            .remove::<HeaderCaseMap>()
            .unwrap_or_default();
        resp.extensions_mut().insert(hyper_headers {
            headers,
            orig_casing,
        });

        hyper_response(resp)
    }
}

unsafe impl AsTaskType for hyper_response {
    fn as_task_type(&self) -> hyper_task_return_type {
        hyper_task_return_type::HYPER_TASK_RESPONSE
    }
}

// ===== impl Headers =====

type hyper_headers_foreach_callback =
    extern "C" fn(*mut c_void, *const u8, size_t, *const u8, size_t) -> c_int;

impl hyper_headers {
    pub(super) fn get_or_default(ext: &mut http::Extensions) -> &mut hyper_headers {
        if let None = ext.get_mut::<hyper_headers>() {
            ext.insert(hyper_headers {
                headers: Default::default(),
                orig_casing: Default::default(),
            });
        }

        ext.get_mut::<hyper_headers>().unwrap()
    }
}

ffi_fn! {
    /// Iterates the headers passing each name and value pair to the callback.
    ///
    /// The `userdata` pointer is also passed to the callback.
    ///
    /// The callback should return `HYPER_ITER_CONTINUE` to keep iterating, or
    /// `HYPER_ITER_BREAK` to stop.
    fn hyper_headers_foreach(headers: *const hyper_headers, func: hyper_headers_foreach_callback, userdata: *mut c_void) {
        let headers = unsafe { &*headers };
        for (name, value) in headers.headers.iter() {
            let (name_ptr, name_len) = if let Some(orig_name) = headers.orig_casing.get(name) {
                (orig_name.as_ptr(), orig_name.len())
            } else {
                (
                    name.as_str().as_bytes().as_ptr(),
                    name.as_str().as_bytes().len(),
                )
            };
            let val_ptr = value.as_bytes().as_ptr();
            let val_len = value.as_bytes().len();

            if HYPER_ITER_CONTINUE != func(userdata, name_ptr, name_len, val_ptr, val_len) {
                break;
            }
        }
    }
}

ffi_fn! {
    /// Sets the header with the provided name to the provided value.
    ///
    /// This overwrites any previous value set for the header.
    fn hyper_headers_set(headers: *mut hyper_headers, name: *const u8, name_len: size_t, value: *const u8, value_len: size_t) -> hyper_code {
        let headers = unsafe { &mut *headers };
        match unsafe { raw_name_value(name, name_len, value, value_len) } {
            Ok((name, value, orig_name)) => {
                headers.headers.insert(&name, value);
                headers.orig_casing.insert(name, orig_name);
                hyper_code::HYPERE_OK
            }
            Err(code) => code,
        }
    }
}

ffi_fn! {
    /// Adds the provided value to the list of the provided name.
    ///
    /// If there were already existing values for the name, this will append the
    /// new value to the internal list.
    fn hyper_headers_add(headers: *mut hyper_headers, name: *const u8, name_len: size_t, value: *const u8, value_len: size_t) -> hyper_code {
        let headers = unsafe { &mut *headers };

        match unsafe { raw_name_value(name, name_len, value, value_len) } {
            Ok((name, value, orig_name)) => {
                headers.headers.append(&name, value);
                headers.orig_casing.append(name, orig_name);
                hyper_code::HYPERE_OK
            }
            Err(code) => code,
        }
    }
}

unsafe fn raw_name_value(
    name: *const u8,
    name_len: size_t,
    value: *const u8,
    value_len: size_t,
) -> Result<(HeaderName, HeaderValue, Bytes), hyper_code> {
    let name = std::slice::from_raw_parts(name, name_len);
    let orig_name = Bytes::copy_from_slice(name);
    let name = match HeaderName::from_bytes(name) {
        Ok(name) => name,
        Err(_) => return Err(hyper_code::HYPERE_INVALID_ARG),
    };
    let value = std::slice::from_raw_parts(value, value_len);
    let value = match HeaderValue::from_bytes(value) {
        Ok(val) => val,
        Err(_) => return Err(hyper_code::HYPERE_INVALID_ARG),
    };

    Ok((name, value, orig_name))
}

// ===== impl HeaderCaseMap =====

impl HeaderCaseMap {
    pub(crate) fn get(&self, name: &HeaderName) -> Option<&Bytes> {
        self.0.get(name)
    }

    pub(crate) fn insert(&mut self, name: HeaderName, orig: Bytes) {
        self.0.insert(name, orig);
    }

    pub(crate) fn append<N>(&mut self, name: N, orig: Bytes)
    where
        N: http::header::IntoHeaderName,
    {
        self.0.append(name, orig);
    }
}
