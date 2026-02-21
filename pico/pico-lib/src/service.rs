use core::cmp::{max, min};

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use atat::heapless::String;
use defmt::info;
use libm::pow;

use crate::call::call_number;
use crate::location::Location;
use crate::poro::{
    CarLocation, ParkLocation, Position, Protector, ProtectorHuman, ProtectorMachine, ReceiverInfo,
    Watcher, WatcherHuman, WatcherMachine,
};
use crate::sms::{SmsStat, read_sms, send_sms};
use crate::utils::{astring_to_string, get_distance_in_meters, is_distance_big_enough};
use crate::{battery, call, gps, network, sms};

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
    // Debug park location updates, car theft etc (with the Watcher Application)
    pub debug_alerts: bool,
    // Send low battery alert
    pub battery_alerts: bool,
    // Detect parking
    pub detect_parking: bool,

    // Keep only the newest N SMS message
    pub keep_n_sms: u32,
}

pub struct DeviceStatus {
    pub last_big_location_change: i64,
    pub location: Option<Location>,
    pub park_location: Option<Location>,
    pub battery: f32,
    pub last_battery_alert: i64,
}

pub struct Service {
    // todo: persistence layer for cfg/status
    pub cfg: Configuration,
    pub status: DeviceStatus,
}

const FIVE_MINUTES_IN_MILLIS: u64 = 5 * 60 * 1_000;
const TWENTY_FIVE_MINUTES_IN_MILLIS: u64 = 5 * FIVE_MINUTES_IN_MILLIS;
const MINIMUM_PARK_LOCATION_ACCURACY_IN_METERS: f64 = 150.0;

impl Service {
    pub async fn init<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
    ) {
        info!("######### CONFIGURATION #########");
        info!("  phone_number: {}", &self.cfg.phone_number);
        info!("  sms_password: {}", &self.cfg.sms_password);
        info!("");
        info!("  service_enabled: {}", self.cfg.service_enabled);
        info!("  locator_poll_count: {}", &self.cfg.locator_poll_count);
        info!("  check_period_seconds: {}", &self.cfg.check_period_seconds);
        info!("");
        info!("  call_after_boot: {}", &self.cfg.call_after_boot);
        info!("  debug_alerts: {}", &self.cfg.debug_alerts);
        info!("  battery_alerts: {}", &self.cfg.battery_alerts);
        info!("  detect_parking: {}", &self.cfg.detect_parking);
        info!("");
        info!("  keep_n_sms: {}", &self.cfg.keep_n_sms);
        info!("#################################");

        // 3x long flash after boot
        for _ in 0..3 {
            pico.set_led_high();
            pico.sleep(500).await;
            pico.set_led_low();
            pico.sleep(500).await;
        }

        info!("Service: init network");
        network::init(client, pico).await;
        info!("Service: init call");
        call::init(client, pico).await;
        info!("Service: init sms");
        sms::init(client, pico).await;
        info!("Service: init time");
        //time::init

        // 10x short flash after init
        for _ in 0..10 {
            pico.set_led_high();
            pico.sleep(100).await;
            pico.set_led_low();
            pico.sleep(100).await;
        }

        if self.cfg.call_after_boot {
            self.call_paired_phone(client, pico).await;
        }
    }

    pub async fn handle_incoming_call<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
        phone_number: &String<30>,
    ) {
        if &self.cfg.phone_number == phone_number {
            pico.sleep(2000).await;
            call::answer_incoming_call(client, pico).await;
        } else {
            call::hangup_incoming_call(client, pico).await;
        }
    }

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

                    if let Some(r) = &w.refresh {
                        if r.value {
                            self.update_battery(client, pico).await;
                            self.refresh(client, pico).await;
                        }
                    }

                    if let Some(p) = w.park {
                        if p.value {
                            if let Some(location) = &self.status.location {
                                self.save_park_location(
                                    client,
                                    pico,
                                    &Location {
                                        latitude: location.latitude,
                                        longitude: location.longitude,
                                        accuracy: location.accuracy,
                                        unix_timestamp_millis: location.unix_timestamp_millis,
                                    },
                                )
                                .await;
                            } else {
                                self.clear_park_location(client, pico).await;
                            }
                        } else {
                            self.clear_park_location(client, pico).await;
                        }
                    }

                    if let Some(r) = w.refresh {
                        if r.value {
                            self.send_message(client, pico, &phone_number, &receiver.source, None)
                                .await;
                        }
                    }

                    if let Some(c) = w.call {
                        if c.value {
                            call_number(client, pico, &phone_number, FIVE_MINUTES_IN_MILLIS).await;
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

            const BATTERY_LOW: f32 = 0.25;
            const THREE_HOURS: i64 = 3 * 60 * 60 * 1000;

            // TODO: instead of uptime we need proper rtc time
            if self.status.battery < BATTERY_LOW && self.cfg.battery_alerts {
                let now = pico.uptime_millis();
                if (now - self.status.last_battery_alert) > THREE_HOURS {
                    self.status.last_battery_alert = now;
                    let message = format!("Battery alert {:.2} %!", self.status.battery);
                    send_sms(
                        client,
                        pico,
                        &self.cfg.phone_number,
                        &astring_to_string::<160>(&message),
                    )
                    .await;
                }
            }
        }
    }

    pub async fn refresh<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
    ) {
        info!("Service: trying to update location");

        // todo fallback to gsm position
        let new_location = gps::get_gps_location(client, pico, self.cfg.locator_poll_count).await;
        if new_location.is_none() {
            return;
        }
        let mut new_location = new_location.unwrap();

        info!("GPS location received {}", new_location);

        let mut accuracy = new_location.accuracy;
        // TODO: do we need this? (my old android code with 68th percentile)
        // We define accuracy as the radius of 68% confidence.
        accuracy = accuracy / 0.68;
        // Unfortunately locations are not reliable when the car is in a garage.
        // This math.pow will try to reduce false alarms.
        // 10 meters -> 15~, 100 -> 200~, 3000 -> 10000~
        accuracy = pow(accuracy, 1.15);
        new_location.accuracy = accuracy;

        info!("GPS location after accuracy adjusted {}", new_location);

        if let Some(p) = &self.status.park_location {
            if is_distance_big_enough(&new_location, &p) {
                // Car Theft detected
                self.send_debug_message(client, pico, Some(crate::poro::Status::CarTheftDetected))
                    .await;
                self.clear_park_location(client, pico).await;
                self.call_paired_phone(client, pico).await;
            } else {
                // Update park location (accuracy might have improved)
                //
                // Should not break the location based car theft detection when the car is moved slowly.
                // The maximum amount of car movement in meters before the park location is not
                // updated anymore is less then park_start_accuracy * 2.
                //
                //  max_movement < park_start_accuracy * 2
                //
                // input:
                //  M : minimum_park_location_accuracy / 2
                //  X : park_start_accuracy
                //
                // output:
                //   sum X * 2 ^ (1-i), i=1 to log2(X / M)
                //
                // e.g:
                //   sum 600 * 2 ^ (1-i), i=1 to log2(600 / 75)
                //
                // http://www.wolframalpha.com/input/?i=sum+600+*+2+%5E+%281-i%29%2C+i%3D1+to+log2%28600+%2F+75%29
                let distance = get_distance_in_meters(
                    new_location.latitude,
                    new_location.longitude,
                    p.latitude,
                    p.longitude,
                );
                if distance <= p.accuracy {
                    if f64::max(
                        new_location.accuracy * 2f64,
                        MINIMUM_PARK_LOCATION_ACCURACY_IN_METERS,
                    ) < p.accuracy
                    {
                        self.save_park_location(client, pico, &new_location).await;
                        self.send_debug_message(
                            client,
                            pico,
                            Some(crate::poro::Status::ParkingUpdated),
                        )
                        .await;
                    }
                }
            }
        } else if self.cfg.detect_parking {
            if let Some(last_location) = &self.status.location {
                if is_distance_big_enough(&new_location, last_location) {
                    self.status.last_big_location_change = new_location.unix_timestamp_millis;
                } else if new_location.unix_timestamp_millis - self.status.last_big_location_change
                    > TWENTY_FIVE_MINUTES_IN_MILLIS as i64
                {
                    // Parking detected
                    self.save_park_location(client, pico, &new_location).await;
                    self.send_debug_message(
                        client,
                        pico,
                        Some(crate::poro::Status::ParkingDetected),
                    )
                    .await;
                }
            } else {
                self.status.last_big_location_change = new_location.unix_timestamp_millis;
            }
        }

        self.save_location(client, pico, &new_location).await;
    }

    async fn call_paired_phone<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
    ) {
        info!("Service: call paired phone='{}'", &self.cfg.phone_number);
        if self.cfg.phone_number.len() > 0 {
            call_number(client, pico, &self.cfg.phone_number, FIVE_MINUTES_IN_MILLIS).await;
        }
    }

    async fn send_debug_message<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
        status: Option<crate::poro::Status>,
    ) {
        info!(
            "Service: send debug message dbg='{}' phone='{}' status='{}'",
            self.cfg.debug_alerts, &self.cfg.phone_number, status
        );
        if self.cfg.debug_alerts {
            if self.cfg.phone_number.len() > 0 {
                let phone_number = self.cfg.phone_number.clone();
                self.send_message(
                    client,
                    pico,
                    &phone_number,
                    &crate::poro::Source::SmsMachine,
                    status,
                )
                .await;
            }
        }
    }

    async fn send_message<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        client: &mut T,
        pico: &mut U,
        phone_number: &String<30>,
        receiver_type: &crate::poro::Source,
        status: Option<crate::poro::Status>,
    ) {
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
            status: status,
            service: Some(crate::poro::Service {
                value: self.cfg.service_enabled,
            }),
        };

        let message = match receiver_type {
            crate::poro::Source::SmsHuman => {
                let p = ProtectorHuman {};
                p.dump(&protector)
            }
            crate::poro::Source::SmsMachine => {
                let p = ProtectorMachine {};
                let dumped = p.dump(&protector);
                let mut tata_response = "$tATA/".to_string();
                let _ = tata_response.push_str(dumped.as_str());
                tata_response
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

    async fn save_location<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        _client: &mut T,
        _pico: &mut U,
        location: &Location,
    ) {
        self.status.location = Some(Location {
            latitude: location.latitude,
            longitude: location.longitude,
            accuracy: location.accuracy,
            unix_timestamp_millis: location.unix_timestamp_millis,
        });
    }

    async fn save_park_location<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        _client: &mut T,
        _pico: &mut U,
        location: &Location,
    ) {
        self.status.park_location = Some(Location {
            latitude: location.latitude,
            longitude: location.longitude,
            accuracy: f64::max(location.accuracy, MINIMUM_PARK_LOCATION_ACCURACY_IN_METERS),
            unix_timestamp_millis: location.unix_timestamp_millis,
        });
    }

    async fn clear_park_location<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
        &mut self,
        _client: &mut T,
        _pico: &mut U,
    ) {
        self.status.park_location = None
    }
}
