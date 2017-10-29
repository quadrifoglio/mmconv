use std::ffi::CStr;

use ff;
use libc::c_int;

error_chain!{
    errors {
        FFmpegError(msg: String) {
            description("error in one of the ffmpeg libraries")
            display("ffmpeg library error: {}", msg)
        }
    }
}

/// Convert an ffmpeg library error code into an FFmpegError.
pub fn ff(err: c_int) -> Error {
    unsafe {
        let mut buf = vec![0i8; 255];

        if ff::av_strerror(err, buf.as_mut_ptr(), 255) < 0 {
            return Error::from(ErrorKind::FFmpegError(String::from("unknown ffmpeg error")));
        }

        let msg = match CStr::from_ptr(buf.as_ptr()).to_owned().into_string() {
            Ok(msg) => msg,
            Err(_) => String::from("unknown ffmpeg error (invalid error message)"),
        };

        Error::from(ErrorKind::FFmpegError(msg))
    }
}
