use core::num::ParseFloatError;
use core::num::ParseIntError;
use core::str::Utf8Error;

use alloc::format;
use alloc::string::ToString;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless::String;
use fasttime::Date;
use fasttime::DateTime;

use crate::at::NoResponse;
use crate::location;
use crate::utils;
use crate::utils::as_tokens;
use crate::utils::send_command_logged;

// SIM868_Series_GNSS_Application_Note_V1.02.pdf

// 2.1 AT+CGNSPWR GNSS Power Control
// AT+CMGF=[<mode>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CGNSPWR", NoResponse)]
pub struct AtGnssPowerControlWrite {
    pub mode: PowerMode,
}

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum PowerMode {
    TurnOff = 0,
    TurnOn = 1,
}

// 2.3 AT+CGNSINF GNSS navigation information parsed from NMEA sentences
// AT+CGNSINF=[<mode>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CGNSINF", GnssNavigationInformationResponse, parse = parse_gnss_navigation_information)]
pub struct AtGnssNavigationInformationExecute;

// +CGNSINF: <GNSS run status>,<Fix status>,<UTC date & Time>,<Latitude>,<Longitude>,<MSL Altitude>,<Speed Over Ground>,<Course Over Ground>,<Fix Mode>,<Reserved1>,<HDOP>,<PDOP>,<VDOP>,<Reserved2>,<G NSS Satellites in View>,<GNSS Satellites Used>,<GLONASS Satellites Used>,<Reserved3>,<C/N0 max>,<HPA>,<VPA>
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
#[rustfmt::skip]
pub struct GnssNavigationInformationResponse {  // Length         Format
    #[at_arg(position = 0)]
    pub gnss_run_status: GNSSRunStatus,         // 1
    #[at_arg(position = 1)]
    pub fix_status: FixStatus,                  // 1
    #[at_arg(position = 2)]
    pub utc_date_time: String<18>,           // 18             yyyyMMddhhmmss.sss [1980-2039][1-12][1-31][0-23][0-59][0.000-60.999]
    #[at_arg(position = 3)]
    pub latitude: f64,                          // 10             [-90.000000,90.000000]
    #[at_arg(position = 4)]
    pub longitude: f64,                         // 11             [-180.000000,180.000000]
    #[at_arg(position = 5)]
    pub msl_altitude: f64,                      // 8              [-180.000000,180.000000] meters
    #[at_arg(position = 6)]
    pub speed_over_ground: f64,                 // 6              [0,999.999] km/h
    #[at_arg(position = 7)]
    pub course_over_ground: f64,                // 6              [0,360.00] degrees
    #[at_arg(position = 8)]
    pub fix_mode: u8,                           // 1              [0,1,2(reserved)]
    #[at_arg(position = 9)]
    pub reserved1: Option<u8>,                  // 0
    #[at_arg(position = 10)]
    pub hdop: f64,                              // 4              [0,99.9] (Horizontal Dilution of Precision)
    #[at_arg(position = 11)]
    pub pdop: f64,                              // 4              [0,99.9] (Position Dilution of Precision)
    #[at_arg(position = 12)]
    pub vdop: f64,                              // 4              [0,99.9] (Vertical Dilution of Precision)
    #[at_arg(position = 13)]
    pub reserved2: Option<u8>,                  // 0
    #[at_arg(position = 14)]
    pub gps_satellites_in_view: Option<u8>,     // 2              [0,99]
    #[at_arg(position = 15)]
    pub gnss_satellites_used: Option<u8>,       // 2              [0,99]
    #[at_arg(position = 16)]
    pub glonass_satellites_in_view: Option<u8>, // 2              [0,99]
    #[at_arg(position = 17)]
    pub reserved3: Option<u8>,                  // 0
    #[at_arg(position = 18)]
    pub c_n0_max: u8,                           // 2              [0,55] dBHz
    #[at_arg(position = 19)]
    pub hpa: Option<f64>,                       // 6              [0,9999.9] meters  (Horizontal Positional Accuracy) reversed
    #[at_arg(position = 20)]
    pub vpa: Option<f64>,                       // 6              [0,9999.9] meters  (Vertical Positional Accuracy)   reversed
} // 94

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

fn parse_gnss_navigation_information(
    response: &[u8],
) -> Result<GnssNavigationInformationResponse, AtatError> {
    log::debug!("   parse_gnss_navigation_information input: {:?}", response);
    let text = core::str::from_utf8(&response[10..])?; // removes "AT+CGNSINF+", ends with \r\n
    let mut tokens = as_tokens(text.trim_end().to_string(), ",");
    log::debug!("   input: {:?}, len={}", tokens, tokens.len());
    if tokens.len() != 21 {
        return Err(atat::Error::Parse.into());
    }

    let mut resp = GnssNavigationInformationResponse::default();

    match tokens.pop_front().unwrap().as_str() {
        "0" => resp.gnss_run_status = GNSSRunStatus::Off,
        "1" => resp.gnss_run_status = GNSSRunStatus::On,
        _ => {
            return Err(atat::Error::Parse.into());
        }
    };

    match tokens.pop_front().unwrap().as_str() {
        "0" => resp.fix_status = FixStatus::NotFixedPosition,
        "1" => resp.fix_status = FixStatus::FixedPosition,
        _ => {
            return Err(atat::Error::Parse.into());
        }
    };

    resp.utc_date_time = String::try_from(tokens.pop_front().unwrap().as_str())?;
    resp.latitude = tokens.pop_front().unwrap().parse()?;
    resp.longitude = tokens.pop_front().unwrap().parse()?;
    resp.msl_altitude = tokens.pop_front().unwrap().parse()?;
    resp.speed_over_ground = tokens.pop_front().unwrap().parse()?;
    resp.course_over_ground = tokens.pop_front().unwrap().parse()?;
    resp.fix_mode = tokens.pop_front().unwrap().parse()?;
    tokens.pop_front(); // reserved1
    resp.hdop = tokens.pop_front().unwrap().parse()?;
    resp.pdop = tokens.pop_front().unwrap().parse()?;
    resp.vdop = tokens.pop_front().unwrap().parse()?;
    tokens.pop_front(); // reserved2

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: u8 = text.parse()?;
            resp.gps_satellites_in_view = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: u8 = text.parse()?;
            resp.gnss_satellites_used = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: u8 = text.parse()?;
            resp.glonass_satellites_in_view = Some(u);
            Ok(())
        }
    }?;

    tokens.pop_front(); // reserved3

    resp.c_n0_max = tokens.pop_front().unwrap().parse()?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.hpa = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.vpa = Some(u);
            Ok(())
        }
    }?;

    return Ok(resp);
}

#[derive(Debug, Clone, PartialEq, AtatEnum, Default)]
pub enum GNSSRunStatus {
    #[default]
    Off = 0,
    On = 1,
}

#[derive(Debug, Clone, PartialEq, AtatEnum, Default)]
pub enum FixStatus {
    #[default]
    NotFixedPosition = 0,
    FixedPosition = 1,
}

pub async fn get_gps_location<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    pico: &mut U,
    max_retries: u8,
) -> Option<location::Location> {
    send_command_logged(
        client,
        &AtGnssPowerControlWrite {
            mode: PowerMode::TurnOn,
        },
        "AtGnssPowerControlWrite ON".to_string(),
    )
    .await
    .ok();

    // TODO defer { AtGnssPowerControlWrite::TurnOff }; would be better

    let mut location: Option<location::Location> = None;
    for i in 0..max_retries {
        match send_command_logged(
            client,
            &AtGnssNavigationInformationExecute,
            format!("AtGnssNavigationInformationExecute {}", i),
        )
        .await
        {
            Ok(resp) => {
                if resp.utc_date_time.len() != 18 {
                    continue;
                }

                let (year, rest) = resp.utc_date_time.as_str().split_at(4);
                let (month, rest) = rest.split_at(2);
                let (day, rest) = rest.split_at(2);
                let (hour, rest) = rest.split_at(2);
                let (minute, rest) = rest.split_at(2);
                let (second, rest) = rest.split_at(2);
                let (_, millis) = rest.split_at(1);

                let datetime = DateTime {
                    date: Date {
                        year: year.parse().unwrap_or_default(),
                        month: month.parse().unwrap_or_default(),
                        day: day.parse().unwrap_or_default(),
                    },
                    time: fasttime::Time {
                        hour: hour.parse().unwrap_or_default(),
                        minute: minute.parse().unwrap_or_default(),
                        second: second.parse().unwrap_or_default(),
                        nanosecond: millis.parse::<u32>().unwrap_or_default() * 1_000_000u32,
                    },
                };

                location = Some(location::Location {
                    latitude: resp.latitude,
                    longitude: resp.longitude,
                    accuracy: utils::estimate_gps_accuracy(resp.pdop),
                    timestamp: (datetime.unix_timestamp_nanos() / 1_000_000) as i64,
                });
                break;
            }
            Err(_) => (),
        }
        pico.sleep(1000).await;
    }

    send_command_logged(
        client,
        &AtGnssPowerControlWrite {
            mode: PowerMode::TurnOff,
        },
        "AtGnssPowerControlWrite OFF".to_string(),
    )
    .await
    .ok();

    return location;
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use crate::{at, cmd_serialization_tests};

    use super::*;
    use atat::AtatCmd;
    use atat::serde_at;

    cmd_serialization_tests! {
        test_at_gnss_power_control_on: (
            AtGnssPowerControlWrite {
                mode: PowerMode::TurnOn,
            },
            13,
            "AT+CGNSPWR=1\r",
        ),
        test_at_gnss_power_control_off: (
            AtGnssPowerControlWrite {
                mode: PowerMode::TurnOff,
            },
            13,
            "AT+CGNSPWR=0\r",
        ),
        test_at_gnss_navigation_information_execute: (
            AtGnssNavigationInformationExecute,
            11,
            "AT+CGNSINF\r",
        ),
    }

    #[test]
    fn test_at_gnss_navigation_information_responses() {
        at::tests::init_env_logger();
        let cmd = AtGnssNavigationInformationExecute;

        assert_eq!(
            atat::Error::Parse,
            cmd.parse(Ok(b"+CGNSINF: ,,,,\r\n")).err().unwrap(),
        );

        assert_eq!(
            atat::Error::Parse,
            cmd.parse(Ok(b"+CGNSINF: 1,1,20221212120221.123,46.7624859,18.6304591,329.218,2.20,285.8,1,,2.1,2.3,0.9,,7,f,,,51,,\r\n")).err().unwrap(),
        );

        assert_eq!(
            GnssNavigationInformationResponse {
                gnss_run_status: GNSSRunStatus::On,
                fix_status: FixStatus::FixedPosition,
                utc_date_time: serde_at::from_slice(b"20221212120221.000").unwrap(),
                latitude: 46.7624859,
                longitude: 18.6304591,
                msl_altitude: 329.218,
                speed_over_ground: 2.20,
                course_over_ground: 285.8,
                fix_mode: 1,
                reserved1: None,
                hdop: 2.1,
                pdop: 2.3,
                vdop: 0.9,
                reserved2: None,
                gps_satellites_in_view: Some(7),
                gnss_satellites_used: Some(6),
                glonass_satellites_in_view: None,
                reserved3: None,
                c_n0_max: 51,
                hpa: None,
                vpa: None,
            },
            cmd.parse(Ok(b"+CGNSINF: 1,1,20221212120221.123,46.7624859,18.6304591,329.218,2.20,285.8,1,,2.1,2.3,0.9,,7,6,,,51,,\r\n")).unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_gps_location() {
        at::tests::init_env_logger();

        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes())); // Turn On
        client.results.push_back(Ok("+CGNSINF: ,,,,".as_bytes())); // error
        client.results.push_back(Ok("+CGNSINF: ,,,,".as_bytes())); // error
        client.results.push_back(Ok("+CGNSINF: 1,1,20221212120221.123,46.7624859,18.6304591,329.218,2.20,285.8,1,,2.1,2.3,0.9,,7,6,,,51,,".as_bytes())); // location
        client.results.push_back(Ok("".as_bytes())); // Turn off

        let mut pico = crate::at::tests::PicoMock::default();
        let loc1 = get_gps_location(&mut client, &mut pico, 5).await;
        assert_eq!(5, client.sent_commands.len());
        assert_eq!("AT+CGNSPWR=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("AT+CGNSINF\r", client.sent_commands.get(1).unwrap());
        assert_eq!("AT+CGNSINF\r", client.sent_commands.get(2).unwrap());
        assert_eq!("AT+CGNSINF\r", client.sent_commands.get(3).unwrap());
        assert_eq!("AT+CGNSPWR=0\r", client.sent_commands.get(4).unwrap());
        assert_eq!(
            location::Location {
                latitude: 46.7624859,
                longitude: 18.6304591,
                accuracy: 5.75,
                timestamp: 1670846541123,
            },
            loc1.unwrap()
        );
        assert_eq!(2, pico.sleep_calls.len());
        assert_eq!(1000, *pico.sleep_calls.get(0).unwrap());
        assert_eq!(1000, *pico.sleep_calls.get(1).unwrap());
    }
}
