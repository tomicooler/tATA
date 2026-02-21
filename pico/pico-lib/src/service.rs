use alloc::string::ToString;
use alloc::vec::Vec;
use atat::heapless::String;
use defmt::info;

use crate::call::call_number;
use crate::location::Location;
use crate::poro::{Protector, ProtectorHuman, ProtectorMachine, ReceiverInfo, Watcher, WatcherHuman, WatcherMachine};
use crate::sms::{SmsStat, read_sms};
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
                            let protector = Protector{
                                car_location: todo!(),
                                park_location: todo!(),
                                status: todo!(),
                                service: todo!(),
                            };

                            match receiver.source {
                                crate::poro::Source::SmsHuman => {
                                    let p = ProtectorHuman{};
                                    p.dump(&protector);
                                },
                                crate::poro::Source::SmsMachine => {
                                    let p = ProtectorMachine{};
                                    p.dump(&protector);
                                },
                                _ => (),
                            }
                        }
                    }

                    if let Some(c) = w.call {
                        if c.value {
                            call_number(
                                client,
                                pico,
                                &astring_to_string::<30>(receiver.phone_number.as_str()),
                                300_000,
                            )
                            .await;
                        }
                    }
                }
            }
        }
    }


    pub async fn refresh<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
    ) {
        gps::get_gps_location(client, pico);
    }
}
