/// Usage example of the mmconv library to convert a video file
/// from MP4/H264 to WebM/VP8.
extern crate mmconv;

use mmconv::io::{Input, Output};
use mmconv::io::stream::StreamKind;

fn main() {
    mmconv::init();

    let mut input = match Input::open("test.in.mp4") {
        Ok(input) => input,
        Err(err) => {
            eprintln!("failed to open input: {}", err);
            return;
        }
    };

    let mut output = Output::create("test.out.webm").unwrap();

    input
        .streams()
        .unwrap()
        .into_iter()
        .filter(|stream| stream.kind() == StreamKind::Video)
        .for_each(|stream| output.add_stream(stream));

    output.process().unwrap();
}
