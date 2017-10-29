/// Usage example of the mmconv library to convert a video file
/// from MP4/H264 to WebM/VP8.
extern crate mmconv;

use mmconv::input::{Input, StreamKind};

fn main() {
    mmconv::init();

    let mut input = match Input::open("test.mp4") {
        Ok(input) => input,
        Err(err) => {
            eprintln!("failed to open input: {}", err);
            return;
        }
    };

    for stream in input.streams().unwrap() {
        let codec = stream.codec();

        match stream.kind() {
            StreamKind::Video => println!("Found video stream: {}", codec),
            StreamKind::Audio => println!("Found audio stream: {}", codec),
            StreamKind::Subs => println!("Found subtitle stream: {}", codec),

            _ => println!("Found stream with unknown type"),
        }
    }
}
