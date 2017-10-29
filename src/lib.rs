/// mmconv library, multimedia conversion functionality for the rust programming language.

#[macro_use]
extern crate error_chain;

extern crate ffmpeg_sys as ff;
extern crate libc;

mod error;

use std::ptr;
use std::ffi::{CStr, CString};

use error::{ErrorKind, Result};

/// Initialize the library.
pub fn init() {
    unsafe {
        ff::av_register_all();
    }
}

/// A multimedia input stream.
pub struct Input {
    pub streams: Vec<Stream>,

    ctx: *mut ff::AVFormatContext,
}

/// A multimedia stream.
pub struct Stream {
    pub index: u32,
    pub kind: StreamKind,

    codec_id: ff::AVCodecID,

    // Decoder context: contains information about the decoded steam.
    // If None, then it means that we won't take interest in that stream.
    dec_ctx: Option<*mut ff::AVCodecContext>,

    // Encoder context: contains information about the re-encoded stream.
    // If None, then it means that we won't take interest in that stream.
    enc_ctx: Option<*mut ff::AVCodecContext>,
}

/// All the different stream types.
pub enum StreamKind {
    Unknown,
    Video,
    Audio,
    Subtitle,
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

            let err = ff::avformat_find_stream_info(ctx, ptr::null_mut());
            if err < 0 {
                ff::avformat_close_input(&mut ctx);
                ff::avformat_free_context(ctx);

                return Err(error::ff(err));
            }

            let nb_streams = (*ctx).nb_streams as isize;
            let mut streams = Vec::with_capacity(nb_streams as usize);

            for i in 0..nb_streams {
                let stream = *(*ctx).streams.offset(i);
                let codec_id = (*(*stream).codecpar).codec_id;

                let decoder = ff::avcodec_find_decoder(codec_id);
                if decoder == ptr::null_mut() {
                    let codec_name = CStr::from_ptr(ff::avcodec_get_name(codec_id))
                        .to_owned()
                        .into_string()
                        .unwrap();

                    ff::avformat_close_input(&mut ctx);
                    ff::avformat_free_context(ctx);

                    return Err(ErrorKind::NoDecoderFound(codec_name).into());
                }

                let mut codec_ctx = ff::avcodec_alloc_context3(decoder);
                if codec_ctx == ptr::null_mut() {
                    return Err(error::ffcustom(
                        "failed to allocate decoder context (AVCodecContext)",
                    ));
                }

                let err = ff::avcodec_parameters_to_context(codec_ctx, (*stream).codecpar);
                if err < 0 {
                    ff::avcodec_free_context(&mut codec_ctx);
                    ff::avformat_close_input(&mut ctx);
                    ff::avformat_free_context(ctx);

                    return Err(error::ff(err));
                }

                let (kind, dec_ctx) = match (*codec_ctx).codec_type {
                    ff::AVMediaType::AVMEDIA_TYPE_VIDEO => (StreamKind::Video, Some(codec_ctx)),
                    ff::AVMediaType::AVMEDIA_TYPE_AUDIO => (StreamKind::Audio, Some(codec_ctx)),
                    ff::AVMediaType::AVMEDIA_TYPE_SUBTITLE => (StreamKind::Subtitle, None),

                    _ => (StreamKind::Unknown, None),
                };

                if dec_ctx.is_some() {
                    (*codec_ctx).framerate = ff::av_guess_frame_rate(ctx, stream, ptr::null_mut());

                    let err = ff::avcodec_open2(codec_ctx, decoder, ptr::null_mut());
                    if err < 0 {
                        ff::avcodec_free_context(&mut codec_ctx);
                        ff::avformat_close_input(&mut ctx);
                        ff::avformat_free_context(ctx);

                        return Err(error::ff(err));
                    }
                }

                streams.push(Stream {
                    index: i as u32,
                    kind: kind,

                    codec_id: codec_id,
                    dec_ctx: dec_ctx,
                    enc_ctx: None,
                });
            }

            Ok(Input {
                streams: streams,
                ctx: ctx,
            })
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            for s in &mut self.streams {
                if let Some(mut dec_ctx) = s.dec_ctx {
                    ff::avcodec_close(dec_ctx);
                    ff::avcodec_free_context(&mut dec_ctx);
                }

                if let Some(mut enc_ctx) = s.enc_ctx {
                    ff::avcodec_close(enc_ctx);
                    ff::avcodec_free_context(&mut enc_ctx);
                }
            }

            ff::avformat_close_input(&mut self.ctx);
            ff::avformat_free_context(self.ctx);
        }
    }
}

impl Stream {
    /// Get the name of the stream's codec.
    pub fn codec_name<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(ff::avcodec_get_name(self.codec_id))
                .to_str()
                .unwrap()
        }
    }
}
