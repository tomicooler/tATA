use crate::{gps::get_gps_location, gsm::get_gsm_location};

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub timestamp: i64,
}

// TODO how to abstract this in rust, a Locator trait Vec<dyn Locator>
pub async fn get_location<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    pico: &mut U,
    max_retries: u8,
    apn: &str,
) -> Option<Location> {
    return get_gps_location(client, pico, max_retries)
        .await
        .or(get_gsm_location(client, pico, max_retries, apn).await);
}
