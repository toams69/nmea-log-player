#![warn(rust_2018_idioms)]
mod nmea_line_codec;
mod settings;

use chrono::NaiveTime;
use futures::{sink::SinkExt, stream::StreamExt};
use nmea_line_codec::{extract_nmea, get_nmea_line_codec};
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin {}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncReadWrite for T {}

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<(), Box<dyn std::error::Error>> {
    // 1
    println!("accept_loop");
    let mut listener = TcpListener::bind(addr).await?; // 2
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        // 3
        // TODO
        println!("new socket opened");
    }
    Ok(())
}

async fn main_loop(
    input_file: String,
    out_pipe: Box<dyn settings::AsyncReadWrite>,
    is_verbose: bool,
) {
    let in_pipe: Box<dyn AsyncReadWrite>;
    in_pipe = Box::new(
        // Open an existing file in read-only
        tokio::fs::File::open(input_file).await.unwrap(),
    );
    let mut reader = get_nmea_line_codec(in_pipe);
    let mut writer = get_nmea_line_codec(out_pipe);
    let mut previous: Option<NaiveTime> = None;
    while let Some(line_result) = reader.next().await {
        let line = line_result.expect("Failed to read line");
        if is_verbose {
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

async fn handleSocket(input_file: String, out_pipe: TcpStream, is_verbose: bool) {
    main_loop(input_file, Box::new(out_pipe), is_verbose);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // First getting and checking settings
    let opts = settings::get_settings().await.unwrap();
    match opts.out_pipe {
        Some(pipe) => {
            main_loop(opts.input_file, pipe, opts.is_verbose).await;
        }
        None => {
            if opts.out_type == "tcp" {
                let addr = format!("{}:{}", opts.out_host, opts.out_port)
                    .parse::<SocketAddr>()
                    .unwrap();
                let mut listener = TcpListener::bind(&addr).await?;
                println!("Listening on: {}", addr);
                loop {
                    let (mut socket, _) = listener.accept().await?;
                    tokio::spawn(async move {
                        handleSocket(opts.input_file, socket, opts.is_verbose).await;
                        // socket
                        //     .write_all((format!("Hello, {}!\n", "Thomas")).as_bytes())
                        //     .await;
                    });
                }
            }
        }
    }
    return Ok(());
}
