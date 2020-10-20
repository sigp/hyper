// We have a lot of c-types in here, stop warning about their names!
#![allow(non_camel_case_types)]

#[macro_use]
mod macros;

mod body;
mod client;
mod http_types;
mod io;
mod task;

#[repr(C)]
pub enum hyper_code {
    HYPERE_OK,
    HYPERE_INVALID_ARG,
}

pub const HYPER_ITER_CONTINUE: libc::c_int = 0;

struct AssertSendSafe<T>(T);

unsafe impl<T> Send for AssertSendSafe<T> {}

/// cbindgen:ignore
static VERSION_CSTR: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

ffi_fn! {
    /// Returns a static ASCII (null terminated) string of the hyper version.
    fn hyper_version() -> *const libc::c_char {
        VERSION_CSTR.as_ptr() as _
    }
}
