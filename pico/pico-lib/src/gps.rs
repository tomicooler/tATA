use atat::serde_at;
use defmt::Format;
use defmt::debug;

use alloc::format;
use alloc::string::ToString;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless_bytes::Bytes;
use fasttime::Date;
use fasttime::DateTime;

use crate::at::NoResponse;
use crate::location;
use crate::utils;
use crate::utils::AtatError;
use crate::utils::as_tokens;
use crate::utils::bytes_to_string;
use crate::utils::send_command_logged;

// SIM868_Series_GNSS_Application_Note_V1.02.pdf

// 2.1 AT+CGNSPWR GNSS Power Control
// AT+CMGF=[<mode>]
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CGNSPWR", NoResponse)]
pub struct AtGnssPowerControlWrite {
    pub mode: PowerMode,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum PowerMode {
    TurnOff = 0,
    TurnOn = 1,
}

// 2.3 AT+CGNSINF GNSS navigation information parsed from NMEA sentences
// AT+CGNSINF=[<mode>]
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CGNSINF", GnssNavigationInformationResponse, parse = parse_gnss_navigation_information)] // TODO: this should not need custom parsing
pub struct AtGnssNavigationInformationExecute;

// +CGNSINF: <GNSS run status>,<Fix status>,<UTC date & Time>,<Latitude>,<Longitude>,<MSL Altitude>,<Speed Over Ground>,<Course Over Ground>,<Fix Mode>,<Reserved1>,<HDOP>,<PDOP>,<VDOP>,<Reserved2>,<G NSS Satellites in View>,<GNSS Satellites Used>,<GLONASS Satellites Used>,<Reserved3>,<C/N0 max>,<HPA>,<VPA>
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
#[rustfmt::skip]
pub struct GnssNavigationInformationResponse {  // Length         Format
    #[at_arg(position = 0)]
    pub gnss_run_status: Option<GNSSRunStatus>,         // 1
    #[at_arg(position = 1)]
    pub fix_status: Option<FixStatus>,                  // 1
    #[at_arg(position = 2)]
    pub utc_date_time: Option<Bytes<18>>,            // 18             yyyyMMddhhmmss.sss [1980-2039][1-12][1-31][0-23][0-59][0.000-60.999]
    #[at_arg(position = 3)]
    pub latitude: Option<f64>,                          // 10             [-90.000000,90.000000]
    #[at_arg(position = 4)]
    pub longitude: Option<f64>,                         // 11             [-180.000000,180.000000]
    #[at_arg(position = 5)]
    pub msl_altitude: Option<f64>,                      // 8              [-180.000000,180.000000] meters
    #[at_arg(position = 6)]
    pub speed_over_ground: Option<f64>,                 // 6              [0,999.999] km/h
    #[at_arg(position = 7)]
    pub course_over_ground: Option<f64>,                // 6              [0,360.00] degrees
    #[at_arg(position = 8)]
    pub fix_mode: Option<u8>,                           // 1              [0,1,2(reserved)]
    #[at_arg(position = 9)]
    pub reserved1: Option<u8>,                          // 0
    #[at_arg(position = 10)]
    pub hdop: Option<f64>,                              // 4              [0,99.9] (Horizontal Dilution of Precision)
    #[at_arg(position = 11)]
    pub pdop: Option<f64>,                              // 4              [0,99.9] (Position Dilution of Precision)
    #[at_arg(position = 12)]
    pub vdop: Option<f64>,                              // 4              [0,99.9] (Vertical Dilution of Precision)
    #[at_arg(position = 13)]
    pub reserved2: Option<u8>,                          // 0
    #[at_arg(position = 14)]
    pub gps_satellites_in_view: Option<u8>,             // 2              [0,99]
    #[at_arg(position = 15)]
    pub gnss_satellites_used: Option<u8>,               // 2              [0,99]
    #[at_arg(position = 16)]
    pub glonass_satellites_in_view: Option<u8>,         // 2              [0,99]
    #[at_arg(position = 17)]
    pub reserved3: Option<u8>,                          // 0
    #[at_arg(position = 18)]
    pub c_n0_max: Option<u8>,                           // 2              [0,55] dBHz
    #[at_arg(position = 19)]
    pub hpa: Option<f64>,                               // 6              [0,9999.9] meters  (Horizontal Positional Accuracy) reversed
    #[at_arg(position = 20)]
    pub vpa: Option<f64>,                               // 6              [0,9999.9] meters  (Vertical Positional Accuracy)   reversed
} // 94

fn parse_gnss_navigation_information(
    response: &[u8],
) -> Result<GnssNavigationInformationResponse, AtatError> {
    debug!("   parse_gnss_navigation_information input: {:?}", response);
    const LEN: usize = "CGNSINF+: ".len();
    if response.len() < LEN {
        return Err(atat::Error::Parse.into());
    }
    let text = core::str::from_utf8(&response[LEN..])?; // removes "AT+CGNSINF+", ends with \r\n
    let mut tokens = as_tokens(text.trim_end().to_string(), ",");
    if tokens.len() != 21 {
        return Err(atat::Error::Parse.into());
    }

    let mut resp = GnssNavigationInformationResponse::default();

    match tokens.pop_front().unwrap().as_str() {
        "0" => resp.gnss_run_status = Some(GNSSRunStatus::Off),
        "1" => resp.gnss_run_status = Some(GNSSRunStatus::On),
        _ => (),
    };

    match tokens.pop_front().unwrap().as_str() {
        "0" => resp.fix_status = Some(FixStatus::NotFixedPosition),
        "1" => resp.fix_status = Some(FixStatus::FixedPosition),
        _ => (),
    };

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: Bytes<18> = serde_at::from_str(text)?;
            resp.utc_date_time = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.latitude = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.longitude = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.msl_altitude = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.speed_over_ground = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.course_over_ground = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: u8 = text.parse()?;
            resp.fix_mode = Some(u);
            Ok(())
        }
    }?;

    tokens.pop_front(); // reserved1

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.hdop = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.pdop = Some(u);
            Ok(())
        }
    }?;

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: f64 = text.parse()?;
            resp.vdop = Some(u);
            Ok(())
        }
    }?;

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

    match tokens.pop_front().unwrap().as_str() {
        "" => Ok::<(), AtatError>(()),
        text => {
            let u: u8 = text.parse()?;
            resp.c_n0_max = Some(u);
            Ok(())
        }
    }?;

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

#[derive(Debug, Format, Clone, PartialEq, AtatEnum, Default)]
pub enum GNSSRunStatus {
    #[default]
    Off = 0,
    On = 1,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum, Default)]
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

    for i in 0..max_retries {
        pico.sleep(1000).await;
        match send_command_logged(
            client,
            &AtGnssNavigationInformationExecute,
            format!("AtGnssNavigationInformationExecute {}", i),
        )
        .await
        {
            Ok(resp) => {
                if resp.utc_date_time.is_none()
                    || resp.latitude.is_none()
                    || resp.longitude.is_none()
                {
                    continue;
                }

                let datetime = bytes_to_string(&resp.utc_date_time.unwrap());
                let (year, rest) = datetime.as_str().split_at(4);
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

                send_command_logged(
                    client,
                    &AtGnssPowerControlWrite {
                        mode: PowerMode::TurnOff,
                    },
                    "AtGnssPowerControlWrite OFF".to_string(),
                )
                .await
                .ok();

                let pdop = resp.pdop.unwrap_or(10.0);
                return Some(location::Location {
                    latitude: resp.latitude.unwrap(),
                    longitude: resp.longitude.unwrap(),
                    accuracy: utils::estimate_gps_accuracy(pdop),
                    unix_timestamp_millis: (datetime.unix_timestamp_nanos() / 1_000_000) as i64,
                });
            }
            Err(_) => (),
        }
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

    return None;
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use crate::cmd_serialization_tests;

    use super::*;
    use atat::AtatCmd;
    use atat::serde_at;

    cmd_serialization_tests! {
        test_at_gnss_power_control_on: (
            AtGnssPowerControlWrite {
                mode: PowerMode::TurnOn,
            },
            "AT+CGNSPWR=1\r",
        ),
        test_at_gnss_power_control_off: (
            AtGnssPowerControlWrite {
                mode: PowerMode::TurnOff,
            },
            "AT+CGNSPWR=0\r",
        ),
        test_at_gnss_navigation_information_execute: (
            AtGnssNavigationInformationExecute,
            "AT+CGNSINF\r",
        ),
    }

    #[test]
    fn test_at_gnss_navigation_information_responses() {
        let cmd = AtGnssNavigationInformationExecute;

        assert_eq!(
            atat::Error::Parse,
            cmd.parse(Ok(b"+CGNSINF: 1,1,20221212120221.123,46.7624859,18.6304591,329.218,2.20,285.8,1,,2.1,2.3,0.9,,7,f,,,51,,\r\n")).err().unwrap(),
        );

        assert_eq!(atat::Error::Parse, cmd.parse(Ok(b"+CGNSI")).err().unwrap(),);

        assert_eq!(
            GnssNavigationInformationResponse {
                gnss_run_status: Some(GNSSRunStatus::Off),
                fix_status: None,
                utc_date_time: None,
                latitude: None,
                longitude: None,
                msl_altitude: None,
                speed_over_ground: None,
                course_over_ground: None,
                fix_mode: None,
                reserved1: None,
                hdop: None,
                pdop: None,
                vdop: None,
                reserved2: None,
                gps_satellites_in_view: None,
                gnss_satellites_used: None,
                glonass_satellites_in_view: None,
                reserved3: None,
                c_n0_max: None,
                hpa: None,
                vpa: None,
            },
            cmd.parse(Ok(b"+CGNSINF: 0,,,,,,,,,,,,,,,,,,,,\r\n"))
                .unwrap(),
        );

        assert_eq!(
            GnssNavigationInformationResponse {
                gnss_run_status: Some(GNSSRunStatus::On),
                fix_status: Some(FixStatus::FixedPosition),
                utc_date_time: Some(serde_at::from_slice(b"20221212120221.123").unwrap()),
                latitude: Some(46.7624859),
                longitude: Some(18.6304591),
                msl_altitude: Some(329.218),
                speed_over_ground: Some(2.20),
                course_over_ground: Some(285.8),
                fix_mode: Some(1),
                reserved1: None,
                hdop: Some(2.1),
                pdop: Some(2.3),
                vdop: Some(0.9),
                reserved2: None,
                gps_satellites_in_view: Some(7),
                gnss_satellites_used: Some(6),
                glonass_satellites_in_view: None,
                reserved3: None,
                c_n0_max: Some(51),
                hpa: None,
                vpa: None,
            },
            cmd.parse(Ok(b"+CGNSINF: 1,1,20221212120221.123,46.7624859,18.6304591,329.218,2.20,285.8,1,,2.1,2.3,0.9,,7,6,,,51,,\r\n")).unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_gps_location() {
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
                unix_timestamp_millis: 1670846541123,
            },
            loc1.unwrap()
        );
        assert_eq!(3, pico.sleep_calls.len());
        assert_eq!(1000, *pico.sleep_calls.get(0).unwrap());
        assert_eq!(1000, *pico.sleep_calls.get(1).unwrap());
        assert_eq!(1000, *pico.sleep_calls.get(2).unwrap());
    }

    // TODO test error handling
}
