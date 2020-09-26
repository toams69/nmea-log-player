use std::{str};
use std::collections::HashMap;

#[derive(Clone)]
enum TimeFormat {
    UtcTime,
    UtcTimeMillis,
}

impl TimeFormat {
    fn to_str(&self) -> &str {
        match *self {
            TimeFormat::UtcTime => "%H%M%S",
            TimeFormat::UtcTimeMillis => "%H%M%S%.3f",
        }
    }
}

#[derive(Clone)]
pub struct NMEATimeRepr {
    position: usize,
    formats: Vec<TimeFormat>,
}

pub fn get_nmea_positions() -> HashMap<String, NMEATimeRepr> {
    [
        (
            "GGA".to_string(),
            NMEATimeRepr {
                position: 1,
                formats: vec![TimeFormat::UtcTime, TimeFormat::UtcTimeMillis],
            },
        ),
        (
            "RMC".to_string(),
            NMEATimeRepr {
                position: 1,
                formats: vec![TimeFormat::UtcTime, TimeFormat::UtcTimeMillis],
            },
        ),
        (
            "GST".to_string(),
            NMEATimeRepr {
                position: 1,
                formats: vec![TimeFormat::UtcTime, TimeFormat::UtcTimeMillis],
            },
        ),
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn get_stamp_from_nmea_line(
    line: &str,
    nmea_positions: &HashMap<String, NMEATimeRepr>,
) -> Option<chrono::naive::NaiveTime> {
    let splits = line.split(",");
    let vec: Vec<&str> = splits.collect();
    if let Some(time_repr) = nmea_positions.get(&vec[0][3..]) {
        for format in &time_repr.formats {
            if let Ok(time) =
                chrono::naive::NaiveTime::parse_from_str(vec[time_repr.position], format.to_str())
            {
                return Some(time);
            }
        }
        return None;
    }
    None
}


#[cfg(test)]
mod tests {
    //TODO
}
