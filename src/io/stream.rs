use std::ptr;
use std::ffi::CStr;

use ff;
use error::{self, ErrorKind, Result};

/// Represents a multimedia stream that reside inside an input file.
pub struct Stream {
    pub(super) codec_ctx: *mut ff::AVCodecContext,
}

/// All the different types of multimedia streams.
#[derive(PartialEq)]
pub enum StreamKind {
    Unknown,
    Video,
    Audio,
    Subs,
}

impl Stream {
    /// Retreive stream information, and try to open the associated codec.
    pub(super) unsafe fn from_raw(s: *mut ff::AVStream) -> Result<Stream> {
        let codec_ctx = (*s).codec;

        match (*codec_ctx).codec_type {
            ff::AVMediaType::AVMEDIA_TYPE_VIDEO | ff::AVMediaType::AVMEDIA_TYPE_AUDIO => {
                let dec = ff::avcodec_find_decoder((*codec_ctx).codec_id);
                if dec == ptr::null_mut() {
                    let name = CStr::from_ptr(ff::avcodec_get_name((*codec_ctx).codec_id))
                        .to_owned()
                        .into_string()
                        .unwrap();

                    return Err(ErrorKind::NoDecoderFound(name).into());
                }

                let err = ff::avcodec_open2(codec_ctx, dec, ptr::null_mut());
                if err < 0 {
                    return Err(error::ff(err));
                }
            }

            _ => {}
        }

        Ok(Stream {
            codec_ctx: codec_ctx,
        })
    }

    /// Return the kind of a stream.
    pub fn kind(&self) -> StreamKind {
        unsafe {
            match (*self.codec_ctx).codec_type {
                ff::AVMediaType::AVMEDIA_TYPE_VIDEO => StreamKind::Video,
                ff::AVMediaType::AVMEDIA_TYPE_AUDIO => StreamKind::Audio,
                ff::AVMediaType::AVMEDIA_TYPE_SUBTITLE => StreamKind::Subs,
                _ => StreamKind::Unknown,
            }
        }
    }

    /// Return the name of the codec associated with a stream.
    pub fn codec<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(ff::avcodec_get_name((*self.codec_ctx).codec_id))
                .to_str()
                .unwrap()
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            ff::avcodec_close(self.codec_ctx);
        }
    }
}
