use bytes::{BufMut, BytesMut};
use nmea::Nmea;
use std::io::{Error, ErrorKind};
use std::{io, str};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct NMEALineCodec;

pub fn extract_nmea(line: &str) -> Result<Nmea, Error> {
    let mut nmea = Nmea::new();
    match nmea.parse(&line) {
        Ok(_sentence_type) => {
            return Ok(nmea);
        }
        Err(e) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Can't parse line\r{}\nError: {}", line, e),
            ))
        }
    }
}

/**
 * This decoder decode a frame with the following signature :
 * $XXXXXXXXXX*YY\n
 * With XXXX any chars that will be returned by the codec
 * YY the checksum of what precede the "*".
 * If the checksum is wrong, the line will be discarded.
 */
impl Decoder for NMEALineCodec {
    type Item = String;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(end_line_index) = newline {
            let line = src.split_to(end_line_index + 1);
            if let Some(start_line_index) = line.as_ref().iter().position(|b| *b == b'$') {
                let nmea_line = line.split_at(start_line_index).1;
                if self.checksum(nmea_line) {
                    return match str::from_utf8(nmea_line) {
                        Ok(s) => {
                            return Ok(Some(s.to_string()));
                        }
                        Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
                    };
                }
            }
        }
        Ok(None)
    }
}

impl Encoder for NMEALineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, line: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        println!("-> {:?}", line);
        dst.reserve(line.len() + 2);
        dst.put(line.as_bytes());
        dst.put_u8(b'\r');
        dst.put_u8(b'\n');
        Ok(())
    }
}

impl NMEALineCodec {
    fn checksum(&mut self, _line: &[u8]) -> bool {
        /*let mut sum = 0;
        for c in line {
            sum = sum ^ c;
        };*/
        true
    }
}

impl Default for NMEALineCodec {
    fn default() -> Self {
        NMEALineCodec {}
    }
}

pub fn get_nmea_line_codec<T: AsyncRead + AsyncWrite>(item: T) -> Framed<T, NMEALineCodec> {
    NMEALineCodec.framed(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_we_can_decode_nmea_line_in_a_trivial_case() {
        let mut codec = NMEALineCodec {};
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(b"$TOTO,jjig*45\n");
        let res = codec.decode(&mut bytes);
        match res {
            Ok(Some(line)) => {
                assert_eq!(line, "$TOTO,jjig*45\n");
                assert_eq!(bytes.len(), 0);
            }
            Ok(None) => {
                panic!("Not Parsed");
            }
            Err(_) => {
                panic!("Can't parse");
            }
        }
    }
    #[test]
    fn test_we_can_decode_nmea_line_in_a_complex_case() {
        let mut codec = NMEALineCodec {};
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(b"$TOTO,jj");
        let mut res = codec.decode(&mut bytes);
        match res {
            Ok(Some(_)) => {
                panic!("Must not parse !");
            }
            Ok(None) => {
                assert_eq!(bytes.len(), 8);
            }
            Err(_) => {
                panic!("Can't parse");
            }
        }

        bytes.extend_from_slice(b"iid*45\n$TATA");
        res = codec.decode(&mut bytes);
        match res {
            Ok(Some(line)) => {
                assert_eq!(line, "$TOTO,jjiid*45\n");
                assert_eq!(bytes.len(), 5); // $TATA.
            }
            Ok(None) => {
                panic!("Not Parsed");
            }
            Err(_) => {
                panic!("Can't parse");
            }
        }
    }
}
