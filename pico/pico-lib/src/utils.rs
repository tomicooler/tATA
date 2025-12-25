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
