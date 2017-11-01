/// This module contains the necessary functionality to handle an multimedia
/// input source.

use ff;
use error::Result;

pub struct Input {
    fmt_ctx: ff::FormatContextPtr,
}

impl Input {
    /// Open an input source.
    pub fn open<S: Into<String>>(url: S) -> Result<Input> {
        Ok(Input {
            fmt_ctx: ff::open_input(url.into())?,
        })
    }

    /// Return the list of streams contained in the input source.
    pub fn streams(&self) -> Vec<Stream> {
        ff::get_streams(self.fmt_ctx)
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        ff::dispose_input(self.fmt_ctx);
    }
}

/// All the different kinds of multimedia streams.
pub enum StreamKind {
    Unknown,
    Video,
    Audio,
    Subtitle,
}

/// Represents a multimedia stream in an input source.
pub struct Stream {
    pub kind: StreamKind,
    pub codec_name: String,

    pub(crate) ptr: ff::StreamPtr,
}
