use defmt::Format;

use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use alloc::vec::Vec;
use machine_derive::MachineDumper;
use machine_derive::MachineParser;

use crate::utils;
use alloc::string::String;

use fasttime::DateTime;

// This is a serialization library that should have been a base64(protobuf()) back in 2014.
// Kept it only to make it work with the existing Android Application.

#[cfg(test)]
extern crate std;

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Service {
    pub value: bool,
}

// Protector

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct CarLocation {
    pub position: Position,
    pub accuracy: f32,
    pub battery: f32,
    pub timestamp: i64,
}

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct ParkLocation {
    pub position: Position,
    pub accuracy: f32,
}

#[derive(Debug, Format, Default, PartialEq)]
pub enum Status {
    #[default]
    ParkingDetected,
    ParkingUpdated,
    CarTheftDetected,
}

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Protector {
    pub car_location: Option<CarLocation>,
    pub park_location: Option<ParkLocation>,
    pub status: Option<Status>,
    pub service: Option<Service>,
}

// Watcher

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Call {
    pub value: bool,
}

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Refresh {
    pub value: bool,
}

#[derive(Debug, Format, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Park {
    pub value: bool,
}

#[derive(Debug, Format, Default, PartialEq)]
pub enum Source {
    #[default]
    Gcm,
    SmsHuman,
    SmsMachine,
    Service,
}

#[derive(Debug, Default, PartialEq, MachineParser, MachineDumper)]
pub struct ReceiverInfo {
    pub source: Source,
    pub phone_number: String,
}

#[derive(Debug, PartialEq, Default, MachineParser, MachineDumper)]
pub struct Watcher {
    pub call: Option<Call>,
    pub refresh: Option<Refresh>,
    pub park: Option<Park>,
    pub receiver: Option<ReceiverInfo>,
    pub service: Option<Service>,
}

// Utils

// https://stackoverflow.com/questions/50277050/format-convert-a-number-to-a-string-in-any-base-including-bases-other-than-deci
fn to_str_radix(value: i64, radix: u64) -> String {
    let mut buf = Vec::new();
    let mut value = value;
    let negative = value < 0;
    if negative {
        value *= -1;
    }
    let mut quotient = value as u64;
    loop {
        buf.push(core::char::from_digit((quotient % radix) as u32, radix as u32).unwrap());
        quotient /= radix;
        if quotient <= 0 {
            break;
        }
    }

    if negative {
        buf.push('-');
    }

    return buf.iter().rev().collect();
}

fn escape(text: String) -> String {
    if text.is_empty() {
        return String::from(EMPTY);
    }

    let mut t = text;
    t = t.replace(SPACE, format!("{}{}", SPACE, SPACE).as_str());
    t = t.replace(
        format!("{}{}", DELIMITER, DELIMITER).as_str(),
        format!("{}{}{}{}", SPACE, SPACE, SPACE, SPACE).as_str(),
    );
    t = t.replace(DELIMITER, SPACE);
    t = t.replace(NULLS, format!("{}{}", NULLS, NULLS).as_str());
    t = t.replace(EMPTY, format!("{}{}", EMPTY, EMPTY).as_str());
    return t;
}

fn unescape(text: String) -> String {
    if text == String::from(EMPTY) {
        return String::new();
    }

    let mut t = text;
    t = t.replace(format!("{}{}", SPACE, SPACE).as_str(), SPACE);
    t = t.replace(SPACE, DELIMITER);
    t = t.replace(format!("{}{}", NULLS, NULLS).as_str(), NULLS);
    t = t.replace(format!("{}{}", EMPTY, EMPTY).as_str(), EMPTY);
    return t;
}

// Machine Parsing / Dumping

const DELIMITER: &'static str = " ";
const SPACE: &'static str = "_";
const NULLS: &'static str = "*";
const TRUE: &'static str = "t";
const FALSE: &'static str = "f";
const EMPTY: &'static str = ";";
const F32_PRECISION: f32 = 1000000f32;
const F64_PRECISION: f64 = 10000000000f64;

trait MachineDumper {
    fn x_dump(&self) -> String;
}

trait MachineParser {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str>;
}

impl<T: MachineParser + Default> MachineParser for Option<T> {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        match tokens.front() {
            Some(top) => {
                if top == NULLS {
                    tokens.pop_front();
                    *self = None;
                } else {
                    let mut v = T::default();
                    let res = v.x_parse(tokens);
                    if res.is_ok() {
                        *self = Some(v);
                    }
                    return res;
                }
            }
            None => return Err("no token for Option<T>"),
        }

        return Ok(());
    }
}

impl MachineParser for f64 {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        match tokens.pop_front() {
            Some(token) => match i64::from_str_radix(token.as_str(), 36) {
                Ok(num) => *self = num as f64 / F64_PRECISION,
                Err(_) => return Err("could not parse radix f64"),
            },
            None => return Err("no token for f64"),
        }
        return Ok(());
    }
}

impl MachineParser for f32 {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        match tokens.pop_front() {
            Some(token) => match i64::from_str_radix(token.as_str(), 36) {
                Ok(num) => *self = num as f32 / F32_PRECISION,
                Err(_) => return Err("could not parse radix f32"),
            },
            None => return Err("no token for f32"),
        }
        return Ok(());
    }
}

impl MachineParser for i64 {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        match tokens.pop_front() {
            Some(token) => match i64::from_str_radix(token.as_str(), 36) {
                Ok(num) => *self = num,
                Err(_) => return Err("could not parse radix i64"),
            },
            None => return Err("no token for i64"),
        }
        return Ok(());
    }
}

impl MachineParser for bool {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        match tokens.pop_front() {
            Some(token) => match token.as_str() {
                TRUE => *self = true,
                FALSE => *self = false,
                _ => return Err("could not parse bool"),
            },
            None => return Err("no token for bool"),
        }
        return Ok(());
    }
}

impl Status {
    fn as_int(&self) -> i64 {
        match &self {
            Status::ParkingDetected => 0,
            Status::ParkingUpdated => 1,
            Status::CarTheftDetected => 2,
        }
    }
}

impl MachineParser for Status {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        let mut x = 0i64;
        x.x_parse(tokens)?;

        match x {
            0 => *self = Status::ParkingDetected,
            1 => *self = Status::ParkingUpdated,
            2 => *self = Status::CarTheftDetected,
            _ => return Err("invalid Status"),
        }

        Ok(())
    }
}

impl Source {
    fn as_int(&self) -> i64 {
        match &self {
            Source::Gcm => 0,
            Source::SmsHuman => 1,
            Source::SmsMachine => 2,
            Source::Service => 3,
        }
    }
}

impl MachineParser for Source {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        let mut x = 0i64;
        x.x_parse(tokens)?;

        match x {
            0 => *self = Source::Gcm,
            1 => *self = Source::SmsHuman,
            2 => *self = Source::SmsMachine,
            3 => *self = Source::Service,
            _ => return Err("invalid Source"),
        }

        Ok(())
    }
}

impl MachineParser for String {
    fn x_parse(&mut self, tokens: &mut VecDeque<String>) -> Result<(), &'static str> {
        match tokens.pop_front() {
            Some(token) => {
                *self = unescape(token);
                Ok(())
            }
            None => Err("no token for String"),
        }
    }
}

impl<T: MachineDumper> MachineDumper for Option<T> {
    fn x_dump(&self) -> String {
        match self.as_ref() {
            Some(v) => v.x_dump(),
            None => String::from(NULLS),
        }
    }
}

impl MachineDumper for f64 {
    fn x_dump(&self) -> String {
        return format!("{}", to_str_radix((self * F64_PRECISION) as i64, 36));
    }
}

impl MachineDumper for f32 {
    fn x_dump(&self) -> String {
        return format!("{}", to_str_radix((self * F32_PRECISION) as i64, 36));
    }
}

impl MachineDumper for i64 {
    fn x_dump(&self) -> String {
        return format!("{}", to_str_radix(*self, 36));
    }
}

impl MachineDumper for bool {
    fn x_dump(&self) -> String {
        return format!("{}", if *self { TRUE } else { FALSE });
    }
}

impl MachineDumper for Status {
    fn x_dump(&self) -> String {
        return self.as_int().x_dump();
    }
}

impl MachineDumper for Source {
    fn x_dump(&self) -> String {
        return self.as_int().x_dump();
    }
}

impl MachineDumper for String {
    fn x_dump(&self) -> String {
        return escape(self.clone());
    }
}

// Serialization Helpers

pub struct ProtectorMachine {}

impl ProtectorMachine {
    pub fn dump(&self, o: &Protector) -> String {
        o.x_dump()
    }

    pub fn parse(&self, d: String) -> Result<Protector, &'static str> {
        let mut tokens = utils::as_tokens(d, DELIMITER);
        let mut p = Protector::default();
        p.x_parse(&mut tokens)?;
        Ok(p)
    }
}

pub struct ProtectorHuman {}

impl ProtectorHuman {
    pub fn dump(&self, o: &Protector) -> String {
        let create_maps_link = |loc: &Position| -> String {
            return format!(
                "https://maps.google.com/?q={},{}",
                loc.latitude, loc.longitude
            );
        };
        let format_unix_timestamp_ms = |timestamp: i64| -> String {
            let secs = timestamp / 1000i64;
            let nanos = (timestamp - secs * 1000) as i32 * 1_000_000i32;
            match DateTime::from_unix_timestamp(secs, nanos) {
                Ok(dt) => format!("{}", dt),
                Err(_) => format!("{}millis", timestamp),
            }
        };

        let mut ret = String::new();

        match o.car_location.as_ref() {
            Some(c) => ret.push_str(
                format!(
                    "{maps_link}\n\n{accuracy:.2} meters, {battery:.2} %, {timestamp}\n\n",
                    maps_link = create_maps_link(&c.position),
                    accuracy = c.accuracy,
                    battery = c.battery * 100.0f32,
                    timestamp = format_unix_timestamp_ms(c.timestamp),
                )
                .as_str(),
            ),
            None => (),
        };

        if o.service.is_some() {
            ret.push_str("Service on\n\n");
        }

        match o.park_location.as_ref() {
            Some(p) => match o.car_location.as_ref() {
                Some(c) => ret.push_str(
                    format!(
                        "Park distance {distance:.2} meters\n\n",
                        distance = utils::get_distance_in_meters(
                            c.position.latitude,
                            c.position.longitude,
                            p.position.latitude,
                            p.position.longitude
                        ),
                    )
                    .as_str(),
                ),
                None => ret.push_str(
                    format!(
                        "Last park location\n\n{maps_link}\n\n{accuracy:.2} meters\n\n",
                        maps_link = create_maps_link(&p.position),
                        accuracy = p.accuracy,
                    )
                    .as_str(),
                ),
            },
            None => (),
        }

        return ret;
    }
}

pub struct WatcherHuman {}

impl WatcherHuman {
    pub fn parse(&self, d: String) -> Result<Watcher, &'static str> {
        match d.as_str() {
            "location" => Ok(Watcher {
                call: None,
                refresh: Some(Refresh { value: true }),
                park: None,
                receiver: None,
                service: None,
            }),
            "call" => Ok(Watcher {
                call: Some(Call { value: true }),
                refresh: None,
                park: None,
                receiver: None,
                service: None,
            }),
            "park on" => Ok(Watcher {
                call: None,
                refresh: None,
                park: Some(Park { value: true }),
                receiver: None,
                service: None,
            }),
            "park off" => Ok(Watcher {
                call: None,
                refresh: None,
                park: Some(Park { value: false }),
                receiver: None,
                service: None,
            }),
            "service on" => Ok(Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: None,
                service: Some(Service { value: true }),
            }),
            "service off" => Ok(Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: None,
                service: Some(Service { value: false }),
            }),
            _ => Err("invalid watcher command"),
        }
    }
}

pub struct WatcherMachine {}

impl WatcherMachine {
    pub fn dump(&self, o: &Watcher) -> String {
        o.x_dump()
    }

    pub fn parse(&self, d: String) -> Result<Watcher, &'static str> {
        let mut tokens = utils::as_tokens(d, DELIMITER);
        let mut w = Watcher::default();
        w.x_parse(&mut tokens)?;
        Ok(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POS1: Position = Position {
        latitude: 46.7624859f64,
        longitude: 18.6304591f64,
    };
    const POS2: Position = Position {
        latitude: 47.1258945f64,
        longitude: 17.8372091f64,
    };
    const CAR_LOC: CarLocation = CarLocation {
        position: POS1,
        accuracy: 250.25f32,
        battery: 0.8912f32,
        timestamp: 1670077542109i64,
    };
    const PARK_LOC: ParkLocation = ParkLocation {
        position: POS2,
        accuracy: 500.5f32,
    };

    macro_rules! protector_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() -> Result<(), &'static str> {
                let (expected_human, expected_machine, protector) = $value;

                let ph = ProtectorHuman{};
                assert_eq!(expected_human, ph.dump(&protector));

                let pm = ProtectorMachine{};
                let machine_dumped = pm.dump(&protector);
                assert_eq!(expected_machine, machine_dumped);
                let re_parsed = pm.parse(machine_dumped)?;
                assert_eq!(protector, re_parsed);

                Ok(())
            }
        )*
        }
    }

    protector_tests! {
        test_protector_0: (
            String::from(""),
            String::from("* * * *"),
            Protector{car_location: None, park_location: None, status: None, service: None},
        ),
        test_protector_1: (
            String::from("https://maps.google.com/?q=46.7624859,18.6304591\n\n250.25 meters, 89.12 %, 2022-12-03T14:25:42.109Z\n\n"),
            String::from("5ytnrmgo 2dl4xyfs 44zq4w j3nk lb811qsd * * *"),
            Protector{car_location: Some(CAR_LOC), park_location: None, status: None, service: None},
        ),
        test_protector_2: (
            String::from("Last park location\n\nhttps://maps.google.com/?q=47.1258945,17.8372091\n\n500.50 meters\n\n"),
            String::from("* 60hrep60 29xy4y9k 89zg9s * *"),
            Protector{car_location: None, park_location: Some(PARK_LOC), status: None, service: None},
        ),
        test_protector_3: (
            String::from(""),
            String::from("* * 2 *"),
            Protector{car_location: None, park_location: None, status: Some(Status::CarTheftDetected), service: None},
        ),
        test_protector_4: (
            String::from("Service on\n\n"),
            String::from("* * * t"),
            Protector{car_location: None, park_location: None, status: None, service: Some(Service{value: true})},
        ),
        test_protector_5: (
            String::from("Last park location\n\nhttps://maps.google.com/?q=47.1258945,17.8372091\n\n500.50 meters\n\n"),
            String::from("* 60hrep60 29xy4y9k 89zg9s 1 *"),
            Protector{car_location: None, park_location: Some(PARK_LOC), status: Some(Status::ParkingUpdated), service: None},
        ),
        test_protector_6: (
            String::from("https://maps.google.com/?q=46.7624859,18.6304591\n\n250.25 meters, 89.12 %, 2022-12-03T14:25:42.109Z\n\nService on\n\nPark distance 72519.74 meters\n\n"),
            String::from("5ytnrmgo 2dl4xyfs 44zq4w j3nk lb811qsd 60hrep60 29xy4y9k 89zg9s 0 t"),
            Protector{car_location: Some(CAR_LOC), park_location: Some(PARK_LOC), status: Some(Status::ParkingDetected), service: Some(Service{value: true})},
        ),
    }

    macro_rules! watcher_human_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() -> Result<(), &'static str> {
                let (expected_watcher, command) = $value;

                let wh = WatcherHuman{};
                let parsed = wh.parse(command)?;
                assert_eq!(expected_watcher, parsed);
                Ok(())
            }
        )*
        }
    }

    watcher_human_tests! {
        test_watcher_human_0: (
            Watcher {
                call: None,
                refresh: Some(Refresh { value: true }),
                park: None,
                receiver: None,
                service: None,
            },
            String::from("location"),
        ),
        test_watcher_human_1: (
            Watcher {
                call: Some(Call { value: true }),
                refresh: None,
                park: None,
                receiver: None,
                service: None,
            },
            String::from("call"),
        ),
        test_watcher_human_2: (
            Watcher {
                call: None,
                refresh: None,
                park: Some(Park { value: true }),
                receiver: None,
                service: None,
            },
            String::from("park on"),
        ),
        test_watcher_human_3: (
            Watcher {
                call: None,
                refresh: None,
                park: Some(Park { value: false }),
                receiver: None,
                service: None,
            },
            String::from("park off"),
        ),
        test_watcher_human_4: (
            Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: None,
                service: Some(Service { value: true }),
            },
            String::from("service on"),
        ),
        test_watcher_human_5: (
            Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: None,
                service: Some(Service { value: false }),
            },
            String::from("service off"),
        ),
    }

    macro_rules! watcher_machine_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() -> Result<(), &'static str> {
                let (expected_machine, watcher) = $value;

                let wm = WatcherMachine{};
                let machine_dumped = wm.dump(&watcher);
                assert_eq!(expected_machine, machine_dumped);
                let re_parsed = wm.parse(machine_dumped)?;
                assert_eq!(watcher, re_parsed);

                Ok(())
            }
        )*
        }
    }

    watcher_machine_tests! {
        test_watcher_machine_0: (
            String::from("* * * * *"),
            Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: None,
                service: None,
            },
        ),
        test_watcher_machine_1: (
            String::from("t * * * *"),
            Watcher {
                call: Some(Call { value: true }),
                refresh: None,
                park: None,
                receiver: None,
                service: None,
            },
        ),
        test_watcher_machine_2: (
            String::from("t f * * *"),
            Watcher {
                call: Some(Call { value: true }),
                refresh: Some(Refresh { value: false }),
                park: None,
                receiver: None,
                service: None,
            },
        ),
        test_watcher_machine_3: (
            String::from("* t f * *"),
            Watcher {
                call: None,
                refresh: Some(Refresh { value: true }),
                park: Some(Park { value: false }),
                receiver: None,
                service: None,
            },
        ),
        test_watcher_machine_4: (
            String::from("* * t 2 phonenumber *"),
            Watcher {
                call: None,
                refresh: None,
                park: Some(Park { value: true }),
                receiver: Some(ReceiverInfo { source: Source::SmsMachine, phone_number: String::from("phonenumber") }),
                service: None,
            },
        ),
        test_watcher_machine_5: (
            String::from("* * * 3 phonenumber f"),
            Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: Some(ReceiverInfo { source: Source::Service, phone_number: String::from("phonenumber") }),
                service: Some(Service { value: false }),
            },
        ),
        test_watcher_machine_6: (
            String::from("* * * 0 phonenumber t"),
            Watcher {
                call: None,
                refresh: None,
                park: None,
                receiver: Some(ReceiverInfo { source: Source::Gcm, phone_number: String::from("phonenumber") }),
                service: Some(Service { value: true }),
            },
        ),
        test_watcher_machine_7: (
            String::from("t f t 1 phonenumber f"),
            Watcher {
                call: Some(Call { value: true }),
                refresh: Some(Refresh { value: false }),
                park: Some(Park { value: true }),
                receiver: Some(ReceiverInfo { source: Source::SmsHuman, phone_number: String::from("phonenumber") }),
                service: Some(Service { value: false }),
            },
        ),
    }

    macro_rules! watcher_machine_escape_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() -> Result<(), &'static str> {
                let (expected_machine, text) = $value;

                let watcher = Watcher {
                    call: None,
                    refresh: None,
                    park: None,
                    receiver: Some(ReceiverInfo { source: Source::SmsMachine, phone_number: String::from(text) }),
                    service: None,
                };

                let wm = WatcherMachine{};
                let machine_dumped = wm.dump(&watcher);
                assert_eq!(expected_machine, machine_dumped);
                let re_parsed = wm.parse(machine_dumped)?;
                assert_eq!(watcher, re_parsed);

                Ok(())
            }
        )*
        }
    }

    watcher_machine_escape_tests! {
        test_watcher_escape_0: (
            "* * * 2 ; *",
            "",
        ),
        test_watcher_escape_1: (
            "* * * 2 _ *",
            " ",
        ),
        test_watcher_escape_2: (
            "* * * 2 ____ *",
            "  ",
        ),
        test_watcher_escape_3: (
            "* * * 2 _____ *",
            "   ",
        ),
        test_watcher_escape_4: (
            "* * * 2 ;; *",
            ";",
        ),
        test_watcher_escape_5: (
            "* * * 2 ;;;; *",
            ";;",
        ),
        test_watcher_escape_6: (
            "* * * 2 ;;;;;; *",
            ";;;",
        ),
        test_watcher_escape_7: (
            "* * * 2 ;;;;;;;; *",
            ";;;;",
        ),
        // TODO: escaping _ is not working, check whether it is possible to fix it in backward compatible way
        // test_watcher_escape_8: (
        //     "* * * 2 ; *",
        //     "_",
        // ),
        // test_watcher_escape_9: (
        //     "* * * 2 ; *",
        //     "__",
        // ),
        // test_watcher_escape_10: (
        //     "* * * 2 ; *",
        //     "___",
        // ),
        // test_watcher_escape_11: (
        //     "* * * 2 ; *",
        //     "____",
        // ),
        test_watcher_escape_12: (
            "* * * 2 ** *",
            "*",
        ),
        test_watcher_escape_13: (
            "* * * 2 **** *",
            "**",
        ),
        test_watcher_escape_14: (
            "* * * 2 ****** *",
            "***",
        ),
        test_watcher_escape_15: (
            "* * * 2 ******** *",
            "****",
        ),
    }
}
