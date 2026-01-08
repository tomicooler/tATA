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

pub struct LogBE {
    context: String,
}

impl LogBE {
    pub fn new(context: String) -> Self {
        log::info!("BEGIN {}", context);
        LogBE { context: context }
    }
}

impl Drop for LogBE {
    fn drop(&mut self) {
        log::info!("END {}", self.context);
    }
}

pub async fn send_command_logged<T: atat::asynch::AtatClient, U: atat::AtatCmd>(
    client: &mut T,
    command: &U,
    context: String,
) -> Result<<U as atat::AtatCmd>::Response, atat::Error> {
    let _l = LogBE::new(context);
    let r = client.send(command).await;
    match r.as_ref() {
        Ok(_) => log::info!("  OK"), // TODO: {:?}, v ?
        Err(e) => log::info!("  ERROR: {:?}", e),
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
