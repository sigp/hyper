use libc::size_t;

pub struct hyper_error(crate::Error);

#[repr(C)]
pub enum hyper_code {
    /// All is well.
    HYPERE_OK,
    /// General error, details in the `hyper_error *`.
    HYPERE_ERROR,
    /// A function argument was invalid.
    HYPERE_INVALID_ARG,
}

// ===== impl hyper_error =====

impl hyper_error {
    fn code(&self) -> hyper_code {
        // TODO: add more variants
        hyper_code::HYPERE_ERROR
    }

    fn print_to(&self, dst: &mut [u8]) -> usize {
        use std::io::Write;

        let mut dst = std::io::Cursor::new(dst);

        // A write! error doesn't matter. As much as possible will have been
        // written, and the Cursor position will know how far that is (even
        // if that is zero).
        let _ = write!(dst, "{}", &self.0);
        dst.position() as usize
    }
}

ffi_fn! {
    /// Frees a `hyper_error`.
    fn hyper_error_free(err: *mut hyper_error) {
        drop(unsafe { Box::from_raw(err) });
    }
}

ffi_fn! {
    /// Get an equivalent `hyper_code` from this error.
    fn hyper_error_code(err: *const hyper_error) -> hyper_code {
        unsafe { &*err }.code()
    }
}

ffi_fn! {
    /// Print the details of this error to a buffer.
    ///
    /// The `dst_len` value must be the maximum length that the buffer can
    /// store.
    ///
    /// The return value is number of bytes that were written to `dst`.
    fn hyper_error_print(err: *const hyper_error, dst: *mut u8, dst_len: size_t) -> size_t {
        let dst = unsafe {
            std::slice::from_raw_parts_mut(dst, dst_len)
        };
        unsafe { &*err }.print_to(dst)
    }
}
