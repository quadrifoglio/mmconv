/// mmconv library, multimedia conversion functionality for the rust programming language.

#[macro_use]
extern crate error_chain;

extern crate ffmpeg_sys as sys;
extern crate libc;

mod ff;
mod error;

pub mod input;
pub mod transcode;

/// Initialize the library.
pub fn init() {
    unsafe {
        sys::av_register_all();
    }
}
