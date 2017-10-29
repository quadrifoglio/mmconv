/// Usage example of the mmconv library to convert a video file
/// from MP4/H264 to WebM/VP8.
extern crate mmconv;

use mmconv::{Input, StreamKind};

fn main() {
    mmconv::init();

    let input = match Input::open("test.mp4") {
        Ok(input) => input,
        Err(err) => {
            eprintln!("failed to open input: {}", err);
            return;
        }
    };

    for stream in &input.streams {
        let codec = stream.codec_name();

        match stream.kind {
            StreamKind::Video => println!("Found video stream: {}", codec),
            StreamKind::Audio => println!("Found audio stream: {}", codec),
            StreamKind::Subtitle => println!("Found subtitle stream: {}", codec),

            _ => println!("Found stream with unknown type"),
        }
    }
}
