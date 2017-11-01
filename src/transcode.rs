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
    pub fn new(input: Input, output: Output) -> Transcode {
        Transcode {
            input: input,
            output: output,
        }
    }

    /// Actually transcode the input source.
    pub fn run(&mut self) -> Result<()> {
        let out_fmt_ctx = ff::open_output(self.output.filename.clone())?;

        for stream in self.input.streams() {
            let output_stream = ff::init_output_stream(out_fmt_ctx)?;

            match stream.kind {
                StreamKind::Video | StreamKind::Audio => {}

                _ => {}
            };
        }

        Ok(())
    }
}
