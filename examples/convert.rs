/// Usage example of the mmconv library to convert a video file
/// from MP4/H264 to WebM/VP8.
extern crate mmconv;

use mmconv::input::{Input, StreamKind};

fn main() {
    mmconv::init();

    let mut input = match Input::open("test.in.mp4") {
        Ok(input) => input,
        Err(err) => {
            eprintln!("failed to open input: {}", err);
            return;
        }
    };

    for stream in input.streams() {
        match stream.kind {
            StreamKind::Video => println!("Video Stream - Codec: {}", stream.codec_name),
            StreamKind::Audio => println!("Audio Stream - Codec: {}", stream.codec_name),
            StreamKind::Subtitle => println!("Subtitle Stream - Codec: {}", stream.codec_name),

            _ => println!("Unknown stream type"),
        }
    }
}
