use std::ffi::CStr;

use sys;
use libc::c_int;

error_chain!{
    errors {
        FFmpegError(msg: String) {
            description("error in one of the ffmpeg libraries")
            display("ffmpeg library error: {}", msg)
        }

        NoDecoderFound(codec: String) {
            description("no decoder found on the system")
            display("no decoder found for codec `{}`", codec)
        }
    }
}

/// Convert an ffmpeg library error code into an FFmpegError.
pub fn ff(err: c_int) -> Error {
    unsafe {
        let mut buf = vec![0i8; 255];

        if sys::av_strerror(err, buf.as_mut_ptr(), 255) < 0 {
            return Error::from(ErrorKind::FFmpegError(String::from("unknown ffmpeg error")));
        }

        let msg = match CStr::from_ptr(buf.as_ptr()).to_owned().into_string() {
            Ok(msg) => msg,
            Err(_) => String::from("unknown ffmpeg error (invalid error message)"),
        };

        Error::from(ErrorKind::FFmpegError(msg))
    }
}

/// Construct an FFmpegError with a custom error message.
pub fn ffcustom<S: Into<String>>(msg: S) -> Error {
    Error::from(ErrorKind::FFmpegError(msg.into()))
}
