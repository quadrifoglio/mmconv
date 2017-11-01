use std::ptr;
use std::ffi::{CStr, CString};

use ff;
use error::{self, ErrorKind, Result};

pub(super) type FormatContextPtr = *mut ff::AVFormatContext;
pub(super) type StreamPtr = *mut ff::AVStream;
pub(super) type Codec = ff::AVCodecID;

/// Open a input source and decode the streams that it contains.
pub(super) fn open_input(url: String) -> Result<FormatContextPtr> {
    unsafe {
        let url = CString::new(url).unwrap();
        let mut ctx: *mut ff::AVFormatContext = ptr::null_mut();

        let err = ff::avformat_open_input(&mut ctx, url.as_ptr(), ptr::null_mut(), ptr::null_mut());
        if err < 0 {
            return Err(error::ff(err));
        }

        let err = ff::avformat_find_stream_info(ctx, ptr::null_mut());
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

/// Find and open the decoder for the specified stream, or return an error
/// if no decoder was found.
pub(super) fn open_stream_decoder(stream: StreamPtr) -> Result<()> {
    unsafe {
        let ctx = (*stream).codec;

        match (*ctx).codec_type {
            ff::AVMediaType::AVMEDIA_TYPE_VIDEO | ff::AVMediaType::AVMEDIA_TYPE_AUDIO => {
                let codec = (*ctx).codec_id;

                let dec = ff::avcodec_find_decoder(codec);
                if dec == ptr::null_mut() {
                    let codec_name = String::from(codec_name(codec));
                    return Err(ErrorKind::NoDecoderFound(codec_name).into());
                }

                let err = ff::avcodec_open2(ctx, dec, ptr::null_mut());
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
pub(super) fn get_streams(fmt_ctx: FormatContextPtr) -> Vec<super::Stream> {
    let mut vec = Vec::new();

    unsafe {
        for i in 0..(*fmt_ctx).nb_streams {
            let stream = *(*fmt_ctx).streams.offset(i as isize);
            let codec_ctx = *(*stream).codec;

            let kind = match codec_ctx.codec_type {
                ff::AVMediaType::AVMEDIA_TYPE_VIDEO => super::StreamKind::Video,
                ff::AVMediaType::AVMEDIA_TYPE_AUDIO => super::StreamKind::Audio,
                ff::AVMediaType::AVMEDIA_TYPE_SUBTITLE => super::StreamKind::Subtitle,

                _ => super::StreamKind::Unknown,
            };

            vec.push(super::Stream {
                kind: kind,
                codec_name: codec_name(codec_ctx.codec_id).to_owned(),
                ptr: stream,
            });
        }
    }

    vec
}

/// Return the name of a codec.
pub(super) fn codec_name(codec: Codec) -> &'static str {
    unsafe {
        CStr::from_ptr(ff::avcodec_get_name(codec))
            .to_str()
            .unwrap()
    }
}

/// Free the resources associated to an opended input.
pub(super) fn dispose_input(mut fmt_ctx: FormatContextPtr) {
    unsafe {
        ff::avformat_close_input(&mut fmt_ctx);
    }
}
