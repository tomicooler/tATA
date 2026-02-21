use core::cmp::{max, min};

use alloc::string::ToString;
use alloc::vec::Vec;
use atat::heapless::String;
use defmt::info;
use libm::pow;

use crate::battery;
use crate::call::call_number;
use crate::location::Location;
use crate::poro::{
    CarLocation, ParkLocation, Position, Protector, ProtectorHuman, ProtectorMachine, ReceiverInfo,
    Watcher, WatcherHuman, WatcherMachine,
};
use crate::sms::{SmsStat, read_sms, send_sms};
use crate::utils::astring_to_string;

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
    pub last_refresh: i64,
    pub location: Option<Location>,
    pub park_location: Option<Location>,
    pub battery: f32,
    pub battery_alert: bool,
}

pub struct Service {
    pub cfg: Configuration,
    pub status: DeviceStatus,
}

impl Service {
    pub async fn handle_sms<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
        index: u32,
    ) {
        if let Some(sms) = read_sms(client, pico, index).await.ok() {
            if sms.stat == SmsStat::ReceivedUnread {
                let v: Vec<&str> = sms.message.split('/').collect();
                if v.len() != 3 {
                    return;
                }
                let prefix = v.get(0).unwrap();
                let command = v.get(1).unwrap();
                let password = v.get(2).unwrap();
                if self.cfg.sms_password != *password {
                    info!("invalid password");
                    return;
                }

                let w: Option<Watcher> = match *prefix {
                    "$tATA" => {
                        let p = WatcherHuman {};
                        let w = p.parse(command.to_string());
                        match w {
                            Ok(w) => Some(Watcher {
                                call: w.call,
                                refresh: w.refresh,
                                park: w.park,
                                receiver: Some(ReceiverInfo {
                                    source: crate::poro::Source::SmsHuman,
                                    phone_number: sms.phone_number.to_string(),
                                }),
                                service: w.service,
                            }),
                            Err(e) => {
                                info!("could not parse watcher human {}", e);
                                None
                            }
                        }
                    }
                    "$TATA" => {
                        let p = WatcherMachine {};
                        let w = p.parse(command.to_string());
                        match w {
                            Ok(w) => Some(Watcher {
                                call: w.call,
                                refresh: w.refresh,
                                park: w.park,
                                receiver: Some(ReceiverInfo {
                                    source: crate::poro::Source::SmsMachine,
                                    phone_number: sms.phone_number.to_string(),
                                }),
                                service: w.service,
                            }),
                            Err(e) => {
                                info!("could not parse watcher machine {}", e);
                                None
                            }
                        }
                    }
                    _ => {
                        info!("not a tATA command");
                        None
                    }
                };

                if let Some(w) = w {
                    let receiver = w.receiver.unwrap();
                    let phone_number = astring_to_string::<30>(receiver.phone_number.as_str());

                    if let Some(s) = w.service {
                        self.cfg.service_enabled = s.value;
                    }

                    if let Some(r) = w.refresh.as_ref() {
                        if r.value {
                            self.refresh(client, pico).await;
                        }
                    }

                    if let Some(p) = w.park {
                        if p.value {
                            if let Some(location) = self.status.location.as_ref() {
                                self.status.park_location = Some(Location {
                                    latitude: location.latitude,
                                    longitude: location.longitude,
                                    accuracy: location.accuracy,
                                    unix_timestamp_millis: location.unix_timestamp_millis,
                                });
                            } else {
                                self.status.park_location = None;
                            }
                        } else {
                            self.status.park_location = None;
                        }
                    }

                    if let Some(r) = w.refresh {
                        if r.value {
                            let car_location = match self.status.location.as_ref() {
                                Some(l) => Some(CarLocation {
                                    position: Position {
                                        latitude: l.latitude,
                                        longitude: l.longitude,
                                    },
                                    accuracy: l.accuracy as f32,
                                    battery: self.status.battery,
                                    timestamp: l.unix_timestamp_millis,
                                }),
                                None => None,
                            };

                            let park_location = match self.status.park_location.as_ref() {
                                Some(l) => Some(ParkLocation {
                                    position: Position {
                                        latitude: l.latitude,
                                        longitude: l.longitude,
                                    },
                                    accuracy: l.accuracy as f32,
                                }),
                                None => None,
                            };

                            let protector = Protector {
                                car_location: car_location,
                                park_location: park_location,
                                status: None, // TODO
                                service: Some(crate::poro::Service {
                                    value: self.cfg.service_enabled,
                                }),
                            };

                            let message = match receiver.source {
                                crate::poro::Source::SmsHuman => {
                                    let p = ProtectorHuman {};
                                    p.dump(&protector)
                                }
                                crate::poro::Source::SmsMachine => {
                                    let p = ProtectorMachine {};
                                    p.dump(&protector)
                                }
                                _ => "".to_string(),
                            };

                            if message.len() > 0 {
                                send_sms(
                                    client,
                                    pico,
                                    &phone_number,
                                    &astring_to_string::<160>(message.as_str()),
                                )
                                .await
                            }
                        }
                    }

                    if let Some(c) = w.call {
                        if c.value {
                            call_number(client, pico, &phone_number, 300_000).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn update_battery<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
    ) {
        info!("Service: trying to update battery");
        if let Some(b) = battery::get_battery(client, pico).await {
            self.status.battery = max(0u8, min(100u8, b.bcl)) as f32 / 100.0f32;
            info!("Service: battery updated {}", self.status.battery);
        }
    }

    pub async fn refresh<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
    ) {
        info!("Service: trying to update location");
        if let Some(loc) =
            crate::gps::get_gps_location(client, pico, self.cfg.locator_poll_count).await
        {
            info!("GPS location received {}", loc);

            let mut accuracy = loc.accuracy;
            // TODO: do we need this? (my old android code with 68th percentile)
            // We define accuracy as the radius of 68% confidence.
            accuracy = accuracy / 0.68;
            // Unfortunately locations are not reliable when the car is in a garage.
            // This math.pow will try to reduce false alarms.
            // 10 meters -> 15~, 100 -> 200~, 3000 -> 10000~
            accuracy = pow(accuracy, 1.15);

            self.status.location = Some(Location {
                latitude: loc.latitude,
                longitude: loc.longitude,
                accuracy: accuracy,
                unix_timestamp_millis: loc.unix_timestamp_millis,
            });
        }
    }
}
