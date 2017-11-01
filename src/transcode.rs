/// This module contains the functionality allowing for transcoding of multimedia data.

use ff;
use error::Result;
use input::{Input, StreamKind};

/// Represents the parameters for the desired output.
pub struct Output {
    filename: String,
}

impl Output {
    /// Construct a new object to represent the targeted
    /// result of the transcoding operation.
    pub fn new<S: Into<String>>(filename: S) -> Output {
        Output {
            filename: filename.into(),
        }
    }
}

/// Represents a transcoding operation.
pub struct Transcode {
    input: Input,
    output: Output,
}

impl Transcode {
    /// Initialize a mutlimedia transcoding operation.
    pub fn new(input: Input, output: Output) -> Result<Transcode> {
        let out_fmt_ctx = ff::open_output(output.filename.clone())?;

        for stream in input.streams() {
            let out_stream = ff::init_output_stream(out_fmt_ctx)?;

            match stream.kind {
                StreamKind::Video | StreamKind::Audio => {
                    ff::prepare_transcode_stream(stream.ptr, out_stream)?
                }

                StreamKind::Subtitle => ff::remux_stream(stream.ptr, out_stream)?,

                _ => {}
            };
        }

        Ok(Transcode {
            input: input,
            output: output,
        })
    }

    /// Actually transcode the input source.
    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }
}
