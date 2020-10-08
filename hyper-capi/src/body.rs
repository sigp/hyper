use std::ffi::c_void;
use std::ptr;

use libc::{c_int, size_t};
use hyper::body::{Body, Bytes, HttpBody as _};

use crate::{AssertSendSafe, ITER_CONTINUE, task::Task};


// ===== Body =====

type ForEachFn = extern "C" fn(*mut c_void, *const Bytes) -> c_int;

ffi_fn! {
    fn hyper_body_foreach(body: *mut Body, func: ForEachFn, userdata: *mut c_void) -> *mut Task {
        if body.is_null() {
            return ptr::null_mut();
        }

        let mut body = unsafe { Box::from_raw(body) };
        let userdata = AssertSendSafe(userdata);

        Box::into_raw(Task::boxed(async move {
            while let Some(item) = body.data().await {
                let chunk = item?;
                if ITER_CONTINUE != func(userdata.0, &chunk) {
                    break;
                }
            }
            Ok(())
        }))
    }
}

// ===== Bytes =====

ffi_fn! {
    fn hyper_buf_bytes(buf: *const Bytes) -> *const u8 {
        unsafe { (*buf).as_ptr() }
    }
}

ffi_fn! {
    fn hyper_buf_len(buf: *const Bytes) -> size_t {
        unsafe { (*buf).len() }
    }
}

ffi_fn! {
    fn hyper_buf_free(buf: *mut Bytes) {
        drop(unsafe { Box::from_raw(buf) });
    }
}
