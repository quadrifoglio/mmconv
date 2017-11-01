/// mmconv library, multimedia conversion functionality for the rust programming language.

#[macro_use]
extern crate error_chain;

extern crate ffmpeg_sys as ff;
extern crate libc;

mod error;

pub mod input;
pub mod transcode;

/// Initialize the library.
pub fn init() {
    unsafe {
        ff::av_register_all();
    }
}
