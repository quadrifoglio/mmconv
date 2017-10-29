/// mmconv library, multimedia conversion functionality for the rust programming language.

#[macro_use]
extern crate error_chain;

extern crate ffmpeg_sys as ff;
extern crate libc;

mod error;

pub mod stream;

use std::ptr;
use std::ffi::CString;

use error::Result;

pub use stream::Stream;

/// Initialize the library.
pub fn init() {
    unsafe {
        ff::av_register_all();
    }
}

/// Represents an input stream, opended from a file.
pub struct Input {
    fmt_ctx: *mut ff::AVFormatContext,
}

impl Input {
    /// Open a multimedia input stream from its URL.
    pub fn open<S: Into<String>>(url: S) -> Result<Input> {
        unsafe {
            let url = CString::new(url.into()).unwrap();
            let mut fmt_ctx: *mut ff::AVFormatContext = ptr::null_mut();

            let err = ff::avformat_open_input(
                &mut fmt_ctx,
                url.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if err < 0 {
                return Err(error::ff(err));
            }

            let err = ff::avformat_find_stream_info(fmt_ctx, ptr::null_mut());
            if err < 0 {
                ff::avformat_close_input(&mut fmt_ctx);
                return Err(error::ff(err));
            }

            Ok(Input { fmt_ctx: fmt_ctx })
        }
    }

    /// Open/Demux the multimedia streams contained in that input.
    pub fn streams(&mut self) -> Result<Vec<Stream>> {
        unsafe {
            let nb_streams = (*self.fmt_ctx).nb_streams as isize;
            let mut streams = Vec::with_capacity(nb_streams as usize);

            for i in 0..nb_streams {
                streams.push(Stream::from_raw(*(*self.fmt_ctx).streams.offset(i))?);
            }

            Ok(streams)
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            ff::avformat_close_input(&mut self.fmt_ctx);
        }
    }
}
