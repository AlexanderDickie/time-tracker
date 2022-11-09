use notify_rust::{Notification, Hint};
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source};

pub fn show_notification() {
    let notification = Notification::new()
        .summary("Finished")
        .body("time has expired")
        .icon("thunderbird")
        .appname("time-tracker")
        .timeout(0) // this however is
        .show().unwrap();

    notification.show().unwrap();
}

pub fn play_sound() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("gong.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));
}

