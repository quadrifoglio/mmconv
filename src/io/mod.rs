/// Input/Output module, contains the functionality for the decoding/encoding
/// of multimedia streams.

pub mod stream;

use std::ptr;
use std::ffi::CString;

use ff;

use error::{self, Result};
use self::stream::{Stream, StreamKind};

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

/// Represents an output stream that will be created.
pub struct Output {
    filename: CString,
    streams: Vec<Stream>,
    fmt_ctx: *mut ff::AVFormatContext,
}

impl Output {
    /// Create a new output stream.
    pub fn create<S: Into<String>>(filename: S) -> Result<Output> {
        unsafe {
            let mut fmt_ctx: *mut ff::AVFormatContext = ptr::null_mut();
            let filename = CString::new(filename.into()).unwrap();

            let err = ff::avformat_alloc_output_context2(
                &mut fmt_ctx,
                ptr::null_mut(),
                ptr::null(),
                filename.as_ptr(),
            );

            if err < 0 {
                return Err(error::ff(err));
            }

            Ok(Output {
                filename: filename,
                streams: Vec::new(),
                fmt_ctx: fmt_ctx,
            })
        }
    }

    /// Add a stream to process.
    pub fn add_stream(&mut self, stream: Stream) {
        self.streams.push(stream);
    }

    /// Run the conversion process.
    pub fn process(self) -> Result<()> {
        unsafe {
            for stream in &self.streams {
                let out_stream = ff::avformat_new_stream(self.fmt_ctx, ptr::null_mut());
                if out_stream == ptr::null_mut() {
                    return Err(error::ffcustom("failed to allocate output stream"));
                }

                let dec_ctx = stream.codec_ctx;
                let enc_ctx = (*out_stream).codec;

                match stream.kind() {
                    StreamKind::Subs => remux(dec_ctx, enc_ctx),
                    StreamKind::Video | StreamKind::Audio => Ok(()),
                    StreamKind::Unknown => Err(error::ffcustom("unknown stream, cannot proceed")),
                }?;
            }

            if ((*(*self.fmt_ctx).oformat).flags & ff::AVFMT_NOFILE) == 0 {
                let err = ff::avio_open(
                    &mut (*self.fmt_ctx).pb,
                    self.filename.as_ptr(),
                    ff::AVIO_FLAG_WRITE,
                );

                if err < 0 {
                    return Err(error::ff(err));
                }
            }

            let err = ff::avformat_write_header(self.fmt_ctx, ptr::null_mut());
            if err < 0 {
                return Err(error::ff(err));
            }

            let err = ff::av_write_trailer(self.fmt_ctx);
            if err < 0 {
                return Err(error::ff(err));
            }

            if ((*(*self.fmt_ctx).oformat).flags & ff::AVFMT_NOFILE) == 0 {
                let err = ff::avio_closep(&mut (*self.fmt_ctx).pb);
                if err < 0 {
                    return Err(error::ff(err));
                }
            }
        }

        Ok(())
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        unsafe {
            ff::avformat_free_context(self.fmt_ctx);
        }
    }
}

/// Remux an input stream into an output stream without any processing whatsoever.
unsafe fn remux(dec_ctx: *mut ff::AVCodecContext, enc_ctx: *mut ff::AVCodecContext) -> Result<()> {
    let err = ff::avcodec_copy_context(dec_ctx, enc_ctx);
    if err < 0 {
        return Err(error::ff(err));
    }

    Ok(())
}
