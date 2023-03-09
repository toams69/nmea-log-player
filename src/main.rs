#![warn(rust_2018_idioms)]

mod nmea_reader;
mod settings;

use bus::Bus;
use tokio::net::TcpListener;

use nmea_reader::{get_nmea_positions, get_stamp_from_nmea_line};
use tokio::io::AsyncWriteExt;

use std::error::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::net::SocketAddr;

fn read_file(mut bus: Bus<String>, filename: &str, replay_factor: f64, is_verbose: bool) {
    // Open the file
    let file = File::open(filename).expect("file not found");
    // Create a buffered reader
    let reader = BufReader::new(file);

    let nmea_positions = get_nmea_positions();

    let mut previous_stamp: Option<chrono::NaiveTime> = None;
    // Read the file line by line
    for line in reader.lines().map(|l| l.unwrap()) {
        if is_verbose {
            println!("Next frame: {}", &line);
        }

        if let Some(current_stamp) = get_stamp_from_nmea_line(&line, &nmea_positions) {
            if let Some(previous_stamp) = previous_stamp {
                if current_stamp > previous_stamp {
                    if let Ok(elapsed) = (current_stamp - previous_stamp).to_std() {
                        let sleep_time_ms = elapsed.as_millis() as f64 / replay_factor;
                        let sleep_duration = std::time::Duration::from_millis(sleep_time_ms as u64);
                        if is_verbose {
                            println!(
                                "sleeping for {:?} with a factor of {} = {:?}",
                                elapsed, replay_factor, sleep_duration
                            );
                        }
                        std::thread::sleep(sleep_duration);
                    }
                }
            }
            previous_stamp = Some(current_stamp);
        } else {
            let sleep_time_ms = 1000 as f64 / replay_factor;
            let sleep_duration = std::time::Duration::from_millis(sleep_time_ms as u64);
            if is_verbose {
                println!(
                    "sleeping for {:?} with a factor of {} = {:?}",
                    1000, replay_factor, sleep_duration
                );
            }
            std::thread::sleep(sleep_duration);
        }

        // Add CR LF at the end of the line
        let new_line = line + "\r\n";
        bus.broadcast(new_line);
    }
    if is_verbose {
        println!("End of File");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // First getting and checking settings
    let opts = settings::get_settings().await.unwrap();
    // Create bus
    let bus = Bus::new(1);
    let bus_handle = bus.read_handle();
    let filename = opts.input_file;

    match opts.out_pipe {
        Some(mut pipe) => {
            let is_verbose = opts.is_verbose;
            let replay_factor = opts.replay_factor;
            std::thread::spawn(move || {
                read_file(bus, &filename, replay_factor, is_verbose);
            });
            let mut receiver = bus_handle.add_rx();
            loop {
                match receiver.recv() {
                    Ok(line) => {
                        if let Err(e) = pipe.write_all(line.as_bytes()).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        None => {
            if opts.out_type == "tcp" {
                let addr = format!("{}:{}", opts.out_host, opts.out_port)
                    .parse::<SocketAddr>()
                    .unwrap();
                // Start reader thread
                let is_verbose = opts.is_verbose;
                let replay_factor = opts.replay_factor;
                std::thread::spawn(move || {
                    read_file(bus, &filename, replay_factor, is_verbose);
                });
                let mut listener = TcpListener::bind(&addr).await?;
                loop {
                    let (mut socket, _) = listener.accept().await?;
                    let mut receiver = bus_handle.add_rx();
                    tokio::spawn(async move {
                        loop {
                            match receiver.recv() {
                                Ok(line) => {
                                    if let Err(e) = socket.write_all(line.as_bytes()).await {
                                        eprintln!("failed to write to socket; err = {:?}", e);
                                        return;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
            }
        }
    }
    return Ok(());
}
