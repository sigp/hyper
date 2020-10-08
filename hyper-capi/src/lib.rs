#[macro_use]
mod macros;

mod body;
mod client;
mod http_types;
mod io;
mod task;

#[repr(C)]
pub enum hyper_code {
    Ok = 0,
    Kaboom = 1,
}

const ITER_CONTINUE: libc::c_int = 0;

struct AssertSendSafe<T>(T);

unsafe impl<T> Send for AssertSendSafe<T> {}

static VERSION_CSTR: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

ffi_fn! {
    fn hyper_version() -> *const u8 {
        VERSION_CSTR.as_ptr()
    }
}
