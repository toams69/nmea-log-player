# nmea-log-player

![Version](https://img.shields.io/badge/version-0.3.0-blue.svg?cacheSeconds=2592000)
![Build](https://img.shields.io/badge/build-unknow-black.svg?cacheSeconds=2592000)

A simple tool allowing to play GPS logs in serial port or through tcp

Uses the nmea crate to decode the gps logs. 

The first frame is send at t0 then the frequency of the rest of the frames is based on timestamp present in 
the input frames.


## Usage

nmea-log-player.exe [FLAGS] [OPTIONS] <input_file> <out_type>

FLAGS:

    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Print each read line

OPTIONS:

    -b, --baudrate <baudrate>          Baudrate to use on the serial port [default: 9600]
    -r, --rate <rate>                  Replay factor. To increase or decrease replay speed. [default: 1.0]
        --serial_port <serial_port>    The serial port to use to send the lines
        --tcp_host <tcp_host>          The TCP host [default: 127.0.0.1]
        --tcp_port <tcp_port>          The tcp port [default: 8080]

ARGS:

    <input_file>    The file to read line by line
    <out_type>      The type of output (could be tcp, tcp-client or serial)


### Example
``nmea-log-player.exe --tcp_host 127.0.0.1 --tcp_port 80 .\gps-logs-samples\logs_gps.nmea tcp``

``nmea-log-player.exe --serial_port COM1 .\gps-logs-samples\logs_gps.nmea serial``

## TODO

- [ ] Allow to specify custom nmea parser (should define how to find the date in sentence and how to parse it).
- [x] Allow to specify a speed (this will allow to speed up or down the read of the file).
- [ ] Allow to pause the replay with the space bar
- [ ] Add UDP socket support
