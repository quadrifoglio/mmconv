/// This module groups functions that abstract functionality from the ffmpeg
/// libraries to make it convinient to use from Rust.

use std::ptr;
use std::ffi::{CStr, CString};

use sys;
use error::{self, ErrorKind, Result};

pub(super) type FormatContextPtr = *mut sys::AVFormatContext;
pub(super) type StreamPtr = *mut sys::AVStream;
pub(super) type Codec = sys::AVCodecID;

/// Open a input source and decode the streams that it contains.
pub(super) fn open_input(url: String) -> Result<FormatContextPtr> {
    unsafe {
        let url = CString::new(url).unwrap();
        let mut ctx: *mut sys::AVFormatContext = ptr::null_mut();

        let err =
            sys::avformat_open_input(&mut ctx, url.as_ptr(), ptr::null_mut(), ptr::null_mut());
        if err < 0 {
            return Err(error::ff(err));
        }

        let err = sys::avformat_find_stream_info(ctx, ptr::null_mut());
        if err < 0 {
            dispose_input(ctx);
            return Err(error::ff(err));
        }

        for i in 0..(*ctx).nb_streams {
            match open_stream_decoder(*(*ctx).streams.offset(i as isize)) {
                Ok(_) => {}
                Err(err) => {
                    dispose_input(ctx);
                    return Err(err);
                }
            };
        }

        Ok(ctx)
    }
}

/// Free the resources associated to an opended input.
pub(super) fn dispose_input(mut fmt_ctx: FormatContextPtr) {
    unsafe {
        sys::avformat_close_input(&mut fmt_ctx);
    }
}

/// Find and open the decoder for the specified stream, or return an error
/// if no decoder was found.
pub(super) fn open_stream_decoder(stream: StreamPtr) -> Result<()> {
    unsafe {
        let ctx = (*stream).codec;

        match (*ctx).codec_type {
            sys::AVMediaType::AVMEDIA_TYPE_VIDEO | sys::AVMediaType::AVMEDIA_TYPE_AUDIO => {
                let codec = (*ctx).codec_id;

                let dec = sys::avcodec_find_decoder(codec);
                if dec == ptr::null_mut() {
                    let codec_name = String::from(codec_name(codec));
                    return Err(ErrorKind::NoDecoderFound(codec_name).into());
                }

                let err = sys::avcodec_open2(ctx, dec, ptr::null_mut());
                if err < 0 {
                    return Err(error::ff(err));
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }
}

/// Construct a vector of objects that represent the streams
/// present in an input source.
pub(super) fn get_streams(fmt_ctx: FormatContextPtr) -> Vec<::input::Stream> {
    let mut vec = Vec::new();

    unsafe {
        for i in 0..(*fmt_ctx).nb_streams {
            let stream = *(*fmt_ctx).streams.offset(i as isize);
            let codec_ctx = *(*stream).codec;

            let kind = match codec_ctx.codec_type {
                sys::AVMediaType::AVMEDIA_TYPE_VIDEO => ::input::StreamKind::Video,
                sys::AVMediaType::AVMEDIA_TYPE_AUDIO => ::input::StreamKind::Audio,
                sys::AVMediaType::AVMEDIA_TYPE_SUBTITLE => ::input::StreamKind::Subtitle,

                _ => ::input::StreamKind::Unknown,
            };

            vec.push(::input::Stream {
                kind: kind,
                codec_name: codec_name(codec_ctx.codec_id).to_owned(),
                ptr: stream,
            });
        }
    }

    vec
}

/// Open an output format context.
pub(super) fn open_output(filename: String) -> Result<FormatContextPtr> {
    unsafe {
        let filename = CString::new(filename).unwrap();
        let mut ctx: FormatContextPtr = ptr::null_mut();

        let err = sys::avformat_alloc_output_context2(
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
        let stream = sys::avformat_new_stream(fmt_ctx, ptr::null_mut());
        if stream == ptr::null_mut() {
            return Err(error::ffcustom("failed to allocate output stream"));
        }

        Ok(stream)
    }
}

/// Return the name of a codec.
pub(super) fn codec_name(codec: Codec) -> &'static str {
    unsafe {
        CStr::from_ptr(sys::avcodec_get_name(codec))
            .to_str()
            .unwrap()
    }
}
