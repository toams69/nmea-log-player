#![warn(rust_2018_idioms)]

mod nmea_line_codec;
mod settings;

use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;

use chrono::NaiveTime;
use nmea_line_codec::{extract_nmea, get_nmea_line_codec};
use tokio::io::AsyncWriteExt;

use futures::SinkExt;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::net::SocketAddr;
use std::path::Path;

use tracing::{info, Level};
use tracing_subscriber;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

async fn client_loop(
    input_file: String,
    out_pipe: Box<dyn settings::AsyncReadWrite>,
    is_verbose: bool,
) {
    let mut writer = get_nmea_line_codec(out_pipe);
    let mut previous: Option<NaiveTime> = None;
    if let Ok(lines) = read_lines(input_file) {
        for line in lines {
            if let Ok(nmea_sentence) = line {
                if is_verbose {
                    println!("{}", nmea_sentence);
                }
                match extract_nmea(&nmea_sentence) {
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
                        let _ = writer.send(nmea_sentence).await;
                    }
                    Err(_e) => {
                        let _ = writer.send(nmea_sentence).await;
                    }
                }
            }
        }
    }
}

/// Process an individual chat client
async fn process(
    input_file: String,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut previous: Option<NaiveTime> = None;
    if let Ok(lines) = read_lines(input_file) {
        for line in lines {
            if let Ok(nmea_sentence) = line {
                info!("{}", nmea_sentence);
                match extract_nmea(&nmea_sentence) {
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
                        stream.write_all(&nmea_sentence.as_bytes()).await;
                    }
                    Err(e) => {}
                }
            }
        }
    }
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // a builder for `FmtSubscriber`.
    let subscriber = tracing_subscriber::fmt()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder
        .finish();
    // and sets the constructed `Subscriber` as the default.
    tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");
    // First getting and checking settings
    let opts = settings::get_settings().await.unwrap();
    match opts.out_pipe {
        Some(pipe) => {
            client_loop(opts.input_file, pipe, opts.is_verbose).await;
        }
        None => {
            if opts.out_type == "tcp" {
                info!("TCP server --");
                let addr = format!("{}:{}", opts.out_host, opts.out_port)
                    .parse::<SocketAddr>()
                    .unwrap();
                let file = opts.input_file;
                let mut listener = TcpListener::bind(&addr).await?;
                loop {
                    // Clone a handle to the `Shared` state for the new connection.
                    // tracing::info!("Listening on: {}", addr);
                    let (stream, addr) = listener.accept().await?;
                    let file = String::from(&file);
                    tokio::spawn(async move {
                        tracing::debug!("accepted connection");
                        if let Err(e) = process(file, stream, addr).await {
                            info!("an error occurred; error = {:?}", e);
                        }
                    });
                }
            }
        }
    }
    return Ok(());
}
