use defmt::info;

use core::num::ParseFloatError;
use core::num::ParseIntError;
use core::str::Utf8Error;

use alloc::collections::vec_deque::VecDeque;
use alloc::string::String;
use libm::{asin, cos, pow, sin, sqrt};

// https://stackoverflow.com/questions/27928/calculate-distance-between-two-latitude-longitude-points-haversine-formula
pub fn get_distance_in_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_in_meters = 6371000f64;

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = pow(sin(d_lat / 2.0f64), 2f64)
        + pow(sin(d_lon / 2.0f64), 2f64) * cos(lat1.to_radians()) * cos(lat2.to_radians());
    let c = 2.0f64 * asin(sqrt(a));
    return earth_radius_in_meters * c;
}

// https://gis.stackexchange.com/questions/111004/translating-hdop-pdop-and-vdop-to-metric-accuracy-from-given-nmea-strings
pub fn estimate_gps_accuracy(pdop: f64) -> f64 {
    // Accuracy 2.5m CEP (circular error probable)
    return 2.5 * pdop;
}

pub fn as_tokens(input: String, delimiter: &'static str) -> VecDeque<String> {
    let parts = input.split(delimiter);
    let mut tokens = VecDeque::new();
    for part in parts {
        tokens.push_back(String::from(part));
    }
    return tokens;
}

pub fn bytes_to_string<const N: usize>(
    bytes: &atat::heapless_bytes::Bytes<N>,
) -> atat::heapless::String<N> {
    let mut data = atat::heapless::Vec::<u8, N>::new();
    for c in bytes.into_iter() {
        let _ = data.push(*c);
    }
    return atat::heapless::String::<N>::from_utf8(data).unwrap();
}

pub fn astring_to_string<'a, const N: usize>(string: &'a str) -> atat::heapless::String<N> {
    return atat::heapless::String::<N>::try_from(string).unwrap();
}

extern crate atat;

#[allow(dead_code)] // field `0` is never read, TODO: research
pub struct AtatError(atat::Error);

impl From<ParseFloatError> for AtatError {
    fn from(_: ParseFloatError) -> Self {
        AtatError(atat::Error::Parse)
    }
}

impl From<ParseIntError> for AtatError {
    fn from(_: ParseIntError) -> Self {
        AtatError(atat::Error::Parse)
    }
}

impl From<()> for AtatError {
    fn from(_: ()) -> Self {
        AtatError(atat::Error::Parse)
    }
}

impl From<Utf8Error> for AtatError {
    fn from(_: Utf8Error) -> Self {
        AtatError(atat::Error::Parse)
    }
}

impl From<atat::Error> for AtatError {
    fn from(value: atat::Error) -> Self {
        AtatError(value)
    }
}

impl From<atat::serde_at::de::Error> for AtatError {
    fn from(_: atat::serde_at::de::Error) -> Self {
        AtatError(atat::Error::Parse)
    }
}

pub async fn send_command_logged<T: atat::asynch::AtatClient, U: atat::AtatCmd>(
    client: &mut T,
    command: &U,
    context: String,
) -> Result<<U as atat::AtatCmd>::Response, atat::Error> {
    info!("SENDING COMMAND: {}", context.as_str());
    let r = client.send(command).await;
    match r.as_ref() {
        Ok(_) => info!("  OK"), // TODO: {:?}, v ?
        Err(e) => info!("  ERROR: {:?}", e),
    }
    return r;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_distance_in_meters() {
        assert_eq!(
            72519.74444090424f64,
            get_distance_in_meters(46.7624859f64, 18.6304591f64, 47.1258945f64, 17.8372091f64)
        );
        assert_eq!(
            0f64,
            get_distance_in_meters(46.7624859f64, 18.6304591f64, 46.7624859f64, 18.6304591f64)
        );
    }
}
