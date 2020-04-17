#![warn(rust_2018_idioms)]
mod nmea_line_codec;
mod settings;

use chrono::NaiveTime;
use futures::{sink::SinkExt, stream::StreamExt};
use nmea_line_codec::{extract_nmea, get_nmea_line_codec};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin {}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncReadWrite for T {}

#[tokio::main]
async fn main() {
    // First getting and checking settings
    let opts = settings::get_settings().await.unwrap();
    let in_pipe: Box<dyn AsyncReadWrite>;
    in_pipe = Box::new(
        // Open an existing file in read-only
        tokio::fs::File::open(opts.input_file).await.unwrap(),
    );
    let mut reader = get_nmea_line_codec(in_pipe);
    let mut writer = get_nmea_line_codec(opts.out_pipe);
    let mut previous: Option<NaiveTime> = None;
    while let Some(line_result) = reader.next().await {
        let line = line_result.expect("Failed to read line");
        if opts.is_verbose {
            println!("{}", line);
        }
        match extract_nmea(&line) {
            Ok(nmea) => {
                if let Some(current) = nmea.fix_time {
                    if let Some(previous) = previous {
                        if current > previous {
                            if let Ok(elapsed) = (current - previous).to_std() {
                                std::thread::sleep(elapsed);
                            }
                        }
                    }
                    previous = Some(current);
                }
                let _ = writer.send(line).await;
            }
            Err(_e) => {
                let _ = writer.send(line).await;
            }
        }
    }
}
