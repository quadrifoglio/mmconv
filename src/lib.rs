/// mmconv library, multimedia conversion functionality for the rust programming language.

#[macro_use]
extern crate error_chain;

extern crate ffmpeg_sys as ff;
extern crate libc;

mod error;

use std::ptr;
use std::ffi::CString;

use error::Result;

/// Initialize the library.
pub fn init() {
    unsafe {
        ff::av_register_all();
    }
}

pub struct Input {
    ctx: *mut ff::AVFormatContext,
}

impl Input {
    /// Open an input stream by its URL.
    pub fn open<S: Into<String>>(url: S) -> Result<Input> {
        unsafe {
            let url = CString::new(url.into()).unwrap();
            let mut ctx = ff::avformat_alloc_context();

            let err =
                ff::avformat_open_input(&mut ctx, url.as_ptr(), ptr::null_mut(), ptr::null_mut());

            if err < 0 {
                ff::avformat_close_input(&mut ctx);
                ff::avformat_free_context(ctx);

                return Err(error::ff(err));
            }

            Ok(Input { ctx: ctx })
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            ff::avformat_close_input(&mut self.ctx);
            ff::avformat_free_context(self.ctx);
        }
    }
}
