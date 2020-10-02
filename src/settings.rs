use clap::{crate_version, value_t, App, Arg};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_serial::{Serial, SerialPortSettings};

// Create an empty trait to allow to use an aggregate of traits
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin {}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncReadWrite for T {}

pub struct AppConfig {
  pub out_pipe: Option<Box<dyn AsyncReadWrite>>,
  pub out_type: String,
  pub out_host: String,
  pub out_port: u32,
  pub input_file: String,
  pub replay_factor: f64,
  pub is_verbose: bool,
}

pub fn get_base_app_args(app_name: &str) -> App<'_, '_> {
  App::new(app_name)
    .version(crate_version!())
    .about("Simple NMEA log player")
    .arg(
      Arg::with_name("input_file")
        .help("The file to read line by line")
        .required(true)
        .index(1),
    )
    .arg(
      Arg::with_name("out_type")
        .help("The type of output (could be tcp, tcp-client or serial)")
        .required(true)
        .index(2),
    )
    .arg(
      Arg::with_name("tcp_host")
        .help("The TCP host")
        .long("tcp_host")
        .takes_value(true)
        .default_value("127.0.0.1"),
    )
    .arg(
      Arg::with_name("tcp_port")
        .help("The tcp port")
        .long("tcp_port")
        .takes_value(true)
        .default_value("8080"),
    )
    .arg(
      Arg::with_name("serial_port")
        .help("The serial port to use to send the lines")
        .long("serial_port")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("baudrate")
        .help("Baudrate to use on the serial port")
        .takes_value(true)
        .short("b")
        .long("baudrate")
        .multiple(false)
        .default_value("9600"),
    )
    .arg(
      Arg::with_name("rate")
        .help("Replay factor. To increase or decrease replay speed.")
        .takes_value(true)
        .short("r")
        .long("rate")
        .multiple(false)
        .default_value("1.0"),
    )
    .arg(
      Arg::with_name("verbose")
        .help("Print each read line")
        .takes_value(false)
        .short("v")
        .long("verbose")
        .multiple(false),
    )
}

type SettingsResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_available_serial_ports() -> Result<Vec<String>, Box<dyn std::error::Error>> {
  // put all port_name into an array
  let port_names: Vec<String> = serialport::available_ports()?
    .iter()
    .map(|x| String::from(&x.port_name))
    .collect::<Vec<String>>();
  Ok(port_names)
}

pub async fn get_settings() -> SettingsResult<AppConfig> {
  // Create help and get args
  let matches = get_base_app_args("nmea-log-player").get_matches();
  let out_type = matches.value_of("out_type").unwrap();
  let out_pipe: Option<Box<dyn AsyncReadWrite>>;
  let out_host;
  let out_port;
  let replay_factor = value_t!(matches, "rate", f64).unwrap();

  match out_type {
    "serial" => {
      let port_names = get_available_serial_ports()?;
      let serial_port_name = matches.value_of("serial_port");
      let port_name = match serial_port_name {
        Some(port_name) => {
          if !port_names.contains(&String::from(port_name)) {
            panic!(format!(
              "Port {} not found in available com port",
              port_name
            ))
          }
          String::from(port_name)
        }
        None => panic!(format!("No port name found")),
      };
      // Get baudrate
      let baudrate = value_t!(matches, "baudrate", u32).expect("Failed to parse baudrate");
      let mut settings: SerialPortSettings = Default::default();
      settings.baud_rate = baudrate;
      out_pipe = Some(Box::new(Serial::from_path(&port_name, &settings).unwrap()));
      out_host = String::from("");
      out_port = 0;
    }
    "tcp-client" => {
      let host = value_t!(matches, "tcp_host", String).expect("Failed to parse tcp_host");
      let port = value_t!(matches, "tcp_port", u32).expect("Failed to parse tcp_port");
      let std_stream = std::net::TcpStream::connect(format!("{}:{}", host, port))?;
      let stream = TcpStream::from_std(std_stream)?;
      out_pipe = Some(Box::new(stream));
      out_host = String::from(&host);
      out_port = port;
    }
    "tcp" => {
      out_pipe = None;
      let host = value_t!(matches, "tcp_host", String).expect("Failed to parse tcp_host");
      let port = value_t!(matches, "tcp_port", u32).expect("Failed to parse tcp_port");
      out_host = String::from(&host);
      out_port = port;
    }
    _ => panic!(format!("out_type not supported")),
  }

  let input_file = matches.value_of("input_file").unwrap();
  Ok(AppConfig {
    out_pipe,
    out_host,
    out_port,
    out_type: String::from(out_type),
    input_file: String::from(input_file),
    replay_factor,
    is_verbose: matches.is_present("verbose"),
  })
}
