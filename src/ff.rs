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
pub(super) fn init_output_stream(ctx: FormatContextPtr) -> Result<StreamPtr> {
    unsafe {
        let out_stream = sys::avformat_new_stream(ctx, ptr::null_mut());
        if out_stream == ptr::null_mut() {
            return Err(error::ffcustom("failed to allocate output stream"));
        }

        Ok(out_stream)
    }
}

/// Set the parameters to allow for a transcode of the specified input stream to an output stream.
pub(super) fn prepare_transcode_stream(in_stream: StreamPtr, out_stream: StreamPtr) -> Result<()> {
    unsafe {
        let dec_ctx = (*in_stream).codec;
        let enc_ctx = (*out_stream).codec;

        let encoder = sys::avcodec_find_encoder((*dec_ctx).codec_id);
        if encoder == ptr::null_mut() {
            return Err(
                ErrorKind::NoDecoderFound(codec_name((*dec_ctx).codec_id).to_owned()).into(),
            );
        }

        if (*dec_ctx).codec_type == sys::AVMediaType::AVMEDIA_TYPE_VIDEO {
            (*enc_ctx).width = (*dec_ctx).width;
            (*enc_ctx).height = (*dec_ctx).height;
            (*enc_ctx).sample_aspect_ratio = (*dec_ctx).sample_aspect_ratio;
            (*enc_ctx).time_base = (*dec_ctx).time_base;

            if (*encoder).pix_fmts != ptr::null_mut() {
                (*enc_ctx).pix_fmt = *(*encoder).pix_fmts;
            }
        }
        if (*dec_ctx).codec_type == sys::AVMediaType::AVMEDIA_TYPE_AUDIO {
            (*enc_ctx).sample_rate = (*dec_ctx).sample_rate;
            (*enc_ctx).channel_layout = (*dec_ctx).channel_layout;
            (*enc_ctx).channels = sys::av_get_channel_layout_nb_channels((*enc_ctx).channel_layout);
            (*enc_ctx).sample_fmt = *(*encoder).sample_fmts;
            (*enc_ctx).time_base = sys::AVRational {
                num: 1,
                den: (*enc_ctx).sample_rate,
            };
        }

        let err = sys::avcodec_open2(enc_ctx, encoder, ptr::null_mut());
        if err < 0 {
            return Err(error::ff(err));
        }
    }

    Ok(())
}

/// Set the parameters to allow for a remux of the specified input stream into the output
/// stream without any processing.
pub(super) fn remux_stream(in_stream: StreamPtr, out_stream: StreamPtr) -> Result<()> {
    unsafe {
        let err = sys::avcodec_copy_context((*out_stream).codec, (*in_stream).codec);
        if err < 0 {
            return Err(error::ff(err));
        }
    }

    Ok(())
}

/// Return the name of a codec.
pub(super) fn codec_name(codec: Codec) -> &'static str {
    unsafe {
        CStr::from_ptr(sys::avcodec_get_name(codec))
            .to_str()
            .unwrap()
    }
}
