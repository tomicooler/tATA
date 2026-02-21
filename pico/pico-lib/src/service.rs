use alloc::string::ToString;
use atat::heapless::String;
use defmt::info;

use crate::location::Location;
use crate::sms::AtReadSMSMessagesWrite;
use crate::utils::send_command_logged;

pub struct Configuration {
    // The phone number is used for alerting
    //  - call at start up (charge -> device restart -> call)
    //  - call when the device is moved out from the parking zone
    //  - battery alert
    //  - SMS notifications
    //  - answer incoming call from this number
    pub phone_number: String<30>,
    // Password for the SMS commands
    pub sms_password: String<30>,

    // The device wakes up every check_period_seconds and
    // refreshes the device's location, battery, parking state, etc
    pub service_enabled: bool,
    // How long the to poll for location
    pub locator_poll_count: u8,
    // The period interval to run the service logic
    pub check_period_seconds: u32,

    // Call the phone_number after boot
    pub call_after_boot: bool,
    // Debug park location updates, etc
    pub debug_alerts: bool,
    // Send low battery alert
    pub battery_alerts: bool,
    // Detect parking
    pub detect_parking: bool,

    // Keep only the newest N SMS message
    pub keep_n_sms: u32,
}

pub struct DeviceStatus {
    pub location: Option<Location>,
    pub park_location: Option<Location>,
    pub battery: f32,
    pub battery_alert: bool,
}

pub struct Service {
    pub cfg: Configuration,
    pub status: DeviceStatus,
}

impl Service {}
