/// Usage example of the mmconv library to convert a video file
/// from MP4/H264 to WebM/VP8.
extern crate mmconv;

use mmconv::Input;

fn main() {
    mmconv::init();

    let input = match Input::open("test.mp4") {
        Ok(input) => input,
        Err(err) => {
            eprintln!("failed to open input: {}", err);
            return;
        }
    };
}
