use std::ptr;
use std::ffi::CString;

use ff;
use error::{self, Result};

pub(super) type FormatContextPtr = *mut ff::AVFormatContext;
pub(super) type StreamPtr = *mut ff::AVStream;

/// Open an output format context.
pub(super) fn open_output(filename: String) -> Result<FormatContextPtr> {
    unsafe {
        let filename = CString::new(filename).unwrap();
        let mut ctx: FormatContextPtr = ptr::null_mut();

        let err = ff::avformat_alloc_output_context2(
            &mut ctx,
            ptr::null_mut(),
            ptr::null_mut(),
            filename.as_ptr(),
        );

        if err < 0 {
            return Err(error::ff(err));
        }

        Ok(ctx)
    }
}

/// Allocate and initialize a multimedia output stream.
pub(super) fn init_output_stream(fmt_ctx: FormatContextPtr) -> Result<StreamPtr> {
    unsafe {
        let stream = ff::avformat_new_stream(fmt_ctx, ptr::null_mut());
        if stream == ptr::null_mut() {
            return Err(error::ffcustom("failed to allocate output stream"));
        }

        Ok(stream)
    }
}
