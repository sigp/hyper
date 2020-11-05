// We have a lot of c-types in here, stop warning about their names!
#![allow(non_camel_case_types)]

#[macro_use]
mod macros;

mod body;
mod client;
mod error;
mod http_types;
mod io;
mod task;

pub(crate) use self::body::UserBody;
pub(crate) use self::http_types::HeaderCaseMap;

pub const HYPER_ITER_CONTINUE: libc::c_int = 0;
#[allow(unused)]
pub const HYPER_ITER_BREAK: libc::c_int = 1;

pub const HYPER_HTTP_VERSION_NONE: libc::c_int = 0;
pub const HYPER_HTTP_VERSION_1_0: libc::c_int = 10;
pub const HYPER_HTTP_VERSION_1_1: libc::c_int = 11;
pub const HYPER_HTTP_VERSION_2: libc::c_int = 20;

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
