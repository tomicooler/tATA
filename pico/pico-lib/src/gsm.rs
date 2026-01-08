use alloc::format;
use alloc::string::ToString;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless::String;
use atat::heapless_bytes::Bytes;
use fasttime::Date;
use fasttime::DateTime;

use crate::at::NoResponse;
use crate::location;
use crate::utils::send_command_logged;

// 7.2.1 AT+CGATT Attach or Detach from GPRS Service
// AT+CGATT=[<state>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CGATT", NoResponse, timeout_ms = 7500)]
pub struct AtAttachGPRS {
    pub state: AttachState,
}

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum AttachState {
    Detach = 0,
    Attach = 1,
}

// 8.2.9 AT+CSTT Start Task and Set APN, USER NAME, PASSWORD
// AT+CSTT=<apn>[,<user name>[,<password>]]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CSTT", NoResponse)]
pub struct AtSetApnWrite {
    pub apn: String<50>,
    pub user_name: Option<String<50>>,
    pub password: Option<String<50>>,
}

// 8.2.10 AT+CIICR Bring up Wireless Connection with GPRS or CSD
// AT+CIICR
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CIICR", NoResponse, timeout_ms = 85000)]
pub struct AtBringUpWirelessConnectionExecute;

// 8.2.11 AT+CIFSR Get Local IP Address
// AT+CIFSR
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CIFSR", GetLocalIPAddressResponse)]
pub struct AtGetLocalIPAddressExecute;

#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
pub struct GetLocalIPAddressResponse {
    pub address: String<50>,
}

// 9.2.1 AT+SAPBR Bearer Settings for Applications Based on IP
// AT+SAPBR=<cmd_type>,<cid>[,<ConParamTag>,<ConParamValue>]
// if <cmd_type> == 2 (QueryBearer)
//   +SAPBR: <cid>,<Status>,<IP_Addr>
// if <cmd_type> == 4 (GetBearerParameters)
//   +SAPBR: <ConParamTag>,<ConParamValue>
// Only type 1 (OpenBearer), type 0 (CloseBearer) and type 3 (SetBearerParameter) is used, so ok with NoResponse
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+SAPBR", NoResponse, timeout_ms = 85000)] // 85 sec for 1 (OpenBearer), 65 sec for 0 (CloseBearer)
#[rustfmt::skip]
pub struct AtSetBearerWrite {
    pub cmd_type: CmdType,
    pub cid: u8,                             // connection id
    pub con_param_tag: Option<String<50>>,   // CONTYPE,  APN,     USER,    PWD,     PHONENUM,  RATE
    pub con_param_value: Option<String<64>>, // CSD/GPRS  len(64)  len(32)  len(32)  for CSD    for CSD (2400, 4800, 9600, 14400)
}

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum CmdType {
    CloseBearer = 0,
    OpenBearer = 1,
    QueryBearer = 2,
    SetBearerParameters = 3,
    GetBearerParameters = 4,
}

// SIM800_Series_GSM_Location_Application_Note_V1.03.pdf

// 2.2 AT+CLBSCFG Base station Location Configuration
// AT+CLBSCFG=<operate>,<para>[,<value>]
// NoResponse for Operate 1 (Set)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLBSCFG", NoResponse)]
pub struct AtBaseStationLocationConfWrite {
    pub operate: Operate,
    pub para: Para,
    pub value: Option<String<50>>,
}

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum Operate {
    Read = 0,
    Set = 1,
}

#[derive(Debug, Clone, PartialEq, AtatEnum, Default)]
pub enum Para {
    #[default]
    CustomerID = 0,
    TimesHaveUsedPositioningCommand = 1,
    ServerAddress = 3, // lbs-simcom.com:3001 lbs-simcom.com:3000 lbs-simcom.com:3002 (default, free)
}

// 2.1 AT+CLBS Base station Location
// AT+CLBS=<type>,<cid>,[[<longitude>,<latitude>],[<lon_type>]]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLBS", BaseStationLocationResponseType4)] // NOTE: only type 4 response is implemented
pub struct AtBaseStationLocationWrite {
    pub type_: LocationType,
    pub cid: u8,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub lon_type: Option<LonType>,
}

#[derive(Debug, Clone, PartialEq, AtatEnum, Default)]
pub enum LocationType {
    #[default]
    Use3Cell = 1,
    GetAccessTimes = 3,
    GetLongLatDateTime = 4,
    ReportPositionError = 9,
}

#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
// +CLBS: <locationcode>[,<longitude>,<latitude>,<acc>,<date>,<time>]
#[rustfmt::skip]
pub struct BaseStationLocationResponseType4 {
    #[at_arg(position = 0)]
    pub location_code: LocationCode,
    #[at_arg(position = 1)]
    pub longitude: Option<f64>,      // [-180.000000,180.000000]
    #[at_arg(position = 2)]
    pub latitude: Option<f64>,       // [-90.000000,90.000000]
    #[at_arg(position = 3)]
    pub accuracy: Option<f64>,
    #[at_arg(position = 4)]
    pub date: Bytes<8>,           // DD/MM/YY
    #[at_arg(position = 5)]
    pub time: Bytes<8>,           // HH:MM:SS
}

#[derive(Debug, Clone, PartialEq, AtatEnum, Default)]
pub enum LocationCode {
    #[default]
    Success = 0,
    Failed = 1,
    Timeout = 2,
    NetError = 3,
    DNSError = 4,
    ServiceOverdue = 5,
    AuthenticationFailed = 6,
    OtherError = 7,
    ReportLbsToServerSuccess = 80,
    ReportLbsToServerParameter = 81,
    ReportLbsToServerFAiled = 82,
}

#[derive(Debug, Clone, PartialEq, AtatEnum, Default)]
pub enum LonType {
    #[default]
    WGS84 = 0,
    GCJ02 = 1, // no plan to launch my car to Mars or China
}

pub async fn get_gsm_location<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    pico: &mut U,
    max_retries: u8,
    apn: &str,
) -> Option<location::Location> {
    let mut loc: Option<location::Location> = None;
    if send_command_logged(
        client,
        &AtAttachGPRS {
            state: AttachState::Attach,
        },
        "AtAttachGPRS ON".to_string(),
    )
    .await
    .is_err()
    {
        return loc;
    }

    // TODO defer { AttachState::Detach }; would be better
    // TODO and_then(), or defer, all failures should stop
    let detach = async |client: &mut T| -> () {
        send_command_logged(
            client,
            &AtAttachGPRS {
                state: AttachState::Detach,
            },
            "AtAttachGPRS OFF".to_string(),
        )
        .await
        .ok();
    };

    if send_command_logged(
        client,
        &AtSetApnWrite {
            apn: String::<50>::try_from(apn).unwrap(),
            user_name: None,
            password: None,
        },
        "AtSetApnWrite".to_string(),
    )
    .await
    .is_err()
    {
        detach(client).await;
        return loc;
    }

    if send_command_logged(
        client,
        &AtBringUpWirelessConnectionExecute,
        "AtBringUpWirelessConnectionExecute".to_string(),
    )
    .await
    .is_err()
    {
        detach(client).await;
        return loc;
    }

    match send_command_logged(
        client,
        &AtGetLocalIPAddressExecute,
        "AtGetLocalIPAddressExecute".to_string(),
    )
    .await
    {
        Ok(v) => log::info!("   OK {:?}", v),
        Err(_) => {
            detach(client).await;
            return loc;
        }
    }

    if send_command_logged(
        client,
        &AtSetBearerWrite {
            cmd_type: CmdType::SetBearerParameters,
            cid: 1,
            con_param_tag: Some(String::<50>::try_from("APN").unwrap()),
            con_param_value: Some(String::<64>::try_from(apn).unwrap()),
        },
        "AtSetBearerWrite APN".to_string(),
    )
    .await
    .is_err()
    {
        detach(client).await;
        return loc;
    }

    if send_command_logged(
        client,
        &AtSetBearerWrite {
            cmd_type: CmdType::SetBearerParameters,
            cid: 1,
            con_param_tag: Some(String::<50>::try_from("Contype").unwrap()),
            con_param_value: Some(String::<64>::try_from("GPRS").unwrap()),
        },
        "AtSetBearerWrite GPRS".to_string(),
    )
    .await
    .is_err()
    {
        detach(client).await;
        return loc;
    }

    if send_command_logged(
        client,
        &AtSetBearerWrite {
            cmd_type: CmdType::OpenBearer,
            cid: 1,
            con_param_tag: None,
            con_param_value: None,
        },
        "AtSetBearerWrite ACTIVATE".to_string(),
    )
    .await
    .is_err()
    {
        detach(client).await;
        return loc;
    }

    let deactivate = async |client: &mut T| -> () {
        send_command_logged(
            client,
            &AtSetBearerWrite {
                cmd_type: CmdType::CloseBearer,
                cid: 1,
                con_param_tag: None,
                con_param_value: None,
            },
            "AtSetBearerWrite DEACTIVATE".to_string(),
        )
        .await
        .ok();
    };

    if send_command_logged(
        client,
        &AtBaseStationLocationConfWrite {
            operate: Operate::Set,
            para: Para::ServerAddress,
            value: Some(String::<50>::try_from("lbs-simcom.com:3002").unwrap()),
        },
        "AtBaseStationLocationConfWrite".to_string(),
    )
    .await
    .is_err()
    {
        deactivate(client).await;
        detach(client).await;
        return loc;
    }

    for i in 0..max_retries {
        match send_command_logged(
            client,
            &AtBaseStationLocationWrite {
                type_: LocationType::GetLongLatDateTime,
                cid: 1,
                longitude: None,
                latitude: None,
                lon_type: None,
            },
            format!("AtBaseStationLocationWrite {}", i),
        )
        .await
        {
            Ok(resp) => {
                log::info!("  OK {:?}", resp);
                if resp.location_code != LocationCode::Success {
                    continue;
                }

                // TODO heapless_bytes::Bytes<8> -> heapless::String<8> conversion, how?
                let mut date_vec = atat::heapless::Vec::<u8, 8>::new();
                for v in resp.date.into_iter() {
                    let _ = date_vec.push(v);
                }
                let date = String::<8>::from_utf8(date_vec).unwrap();
                let (day, rest) = date.as_str().split_at(2);
                let (_, rest) = rest.split_at(1);
                let (month, rest) = rest.split_at(2);
                let (_, rest) = rest.split_at(1);
                let (yy, _) = rest.split_at(2);
                let year = format!("20{}", yy);

                // TODO heapless_bytes::Bytes<8> -> heapless::String<8> conversion, how?
                let mut time_vec = atat::heapless::Vec::<u8, 8>::new();
                for v in resp.time.into_iter() {
                    let _ = time_vec.push(v);
                }
                let time = String::<8>::from_utf8(time_vec).unwrap();
                let (hour, rest) = time.as_str().split_at(2);
                let (_, rest) = rest.split_at(1);
                let (minute, rest) = rest.split_at(2);
                let (_, rest) = rest.split_at(1);
                let (second, _) = rest.split_at(2);

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
                        nanosecond: 0,
                    },
                };

                loc = Some(location::Location {
                    latitude: resp.latitude.unwrap_or_default(),
                    longitude: resp.longitude.unwrap_or_default(),
                    accuracy: resp.accuracy.unwrap_or_default(),
                    timestamp: (datetime.unix_timestamp_nanos() / 1_000_000) as i64,
                });
                break;
            }
            Err(_) => (),
        }
        pico.sleep(1000).await;
    }

    deactivate(client).await;
    detach(client).await;
    return loc;
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
        test_at_attach_gprs_on: (
            AtAttachGPRS {
                state: AttachState::Attach,
            },
            "AT+CGATT=1\r",
        ),
        test_at_attach_gprs_off: (
            AtAttachGPRS {
                state: AttachState::Detach,
            },
            "AT+CGATT=0\r",
        ),
        test_at_set_apn_write: (
            AtSetApnWrite {
                apn: String::<50>::try_from("online").unwrap(),
                user_name: None,
                password: None,
            },
            "AT+CSTT=\"online\"\r",
        ),
        test_at_bring_up_wireless_connection_execute: (
            AtBringUpWirelessConnectionExecute,
            "AT+CIICR\r",
        ),
        test_at_get_local_ip_address_execute: (
            AtGetLocalIPAddressExecute,
            "AT+CIFSR\r",
        ),
        test_at_set_bearer_write_apn: (
            AtSetBearerWrite {
                cmd_type: CmdType::SetBearerParameters,
                cid: 1,
                con_param_tag: Some(String::<50>::try_from("APN").unwrap()),
                con_param_value: Some(String::<64>::try_from("online").unwrap()),
            },
            "AT+SAPBR=3,1,\"APN\",\"online\"\r",
        ),
        test_at_set_bearer_write_gprs: (
            AtSetBearerWrite {
                cmd_type: CmdType::SetBearerParameters,
                cid: 1,
                con_param_tag: Some(String::<50>::try_from("Contype").unwrap()),
                con_param_value: Some(String::<64>::try_from("GPRS").unwrap()),
            },
            "AT+SAPBR=3,1,\"Contype\",\"GPRS\"\r",
        ),
        test_at_set_bearer_write_activate: (
            AtSetBearerWrite {
                cmd_type: CmdType::OpenBearer,
                cid: 1,
                con_param_tag: None,
                con_param_value: None,
            },
            "AT+SAPBR=1,1\r",
        ),
        test_at_set_bearer_write_deactivate: (
            AtSetBearerWrite {
                cmd_type: CmdType::CloseBearer,
                cid: 1,
                con_param_tag: None,
                con_param_value: None,
            },
            "AT+SAPBR=0,1\r",
        ),
        test_at_base_station_conf_write: (
            AtBaseStationLocationConfWrite {
                operate: Operate::Set,
                para: Para::ServerAddress,
                value: Some(String::<50>::try_from("lbs-simcom.com:3002").unwrap()),
            },
            "AT+CLBSCFG=1,3,\"lbs-simcom.com:3002\"\r",
        ),
        test_at_base_station_write: (
            AtBaseStationLocationWrite {
                type_: LocationType::GetLongLatDateTime,
                cid: 1,
                longitude: None,
                latitude: None,
                lon_type: None,
            },
            "AT+CLBS=4,1\r",
        ),
    }

    #[test]
    fn test_at_get_local_ip_address_response() {
        at::tests::init_env_logger();
        let cmd = AtGetLocalIPAddressExecute;

        assert_eq!(
            GetLocalIPAddressResponse {
                address: String::<50>::try_from("100.95.173.97").unwrap(),
            },
            cmd.parse(Ok(b"100.95.173.97\r\n")).unwrap()
        );
    }

    #[test]
    fn test_at_a() {
        at::tests::init_env_logger();
        let cmd = AtBaseStationLocationWrite {
            type_: LocationType::GetLongLatDateTime,
            cid: 1,
            longitude: None,
            latitude: None,
            lon_type: None,
        };

        let date: Bytes<8> = serde_at::from_slice(b"12/12/22").unwrap();
        let time: Bytes<8> = serde_at::from_slice(b"12:02:21").unwrap();
        assert_eq!(
            BaseStationLocationResponseType4 {
                location_code: LocationCode::Success,
                longitude: Some(18.6304591),
                latitude: Some(46.7624859),
                accuracy: Some(550.0),
                date: date,
                time: time,
            },
            cmd.parse(Ok(
                b"+CLBS: 0,18.6304591,46.7624859,550,12/12/22,12:02:21\r\n"
            ))
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_gsm_location() {
        at::tests::init_env_logger();

        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes())); // GPRS On
        client.results.push_back(Ok("".as_bytes())); // Set APN
        client.results.push_back(Ok("".as_bytes())); // Bring up
        client.results.push_back(Ok("100.95.173.97".as_bytes())); // Get Local IP
        client.results.push_back(Ok("".as_bytes())); // Set Bearer APN
        client.results.push_back(Ok("".as_bytes())); // Set Bearer GPRS
        client.results.push_back(Ok("".as_bytes())); // Activate Bearer
        client.results.push_back(Ok("".as_bytes())); // Set LBS Address (free version)
        client.results.push_back(Ok("".as_bytes())); // Get Lat Long
        client.results.push_back(Ok("".as_bytes())); // Get Lat Long
        client.results.push_back(Ok(
            "+CLBS: 0,18.6304591,46.7624859,550,12/12/22,12:02:21".as_bytes()
        )); // location
        client.results.push_back(Ok("".as_bytes())); // Deactivate Bearer
        client.results.push_back(Ok("".as_bytes())); // GPRS off

        let mut pico = crate::at::tests::PicoMock::default();
        let loc1 = get_gsm_location(&mut client, &mut pico, 5, "online").await;
        assert_eq!(13, client.sent_commands.len());
        assert_eq!("AT+CGATT=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("AT+CSTT=\"online\"\r", client.sent_commands.get(1).unwrap());
        assert_eq!("AT+CIICR\r", client.sent_commands.get(2).unwrap());
        assert_eq!("AT+CIFSR\r", client.sent_commands.get(3).unwrap());
        assert_eq!(
            "AT+SAPBR=3,1,\"APN\",\"online\"\r",
            client.sent_commands.get(4).unwrap()
        );
        assert_eq!(
            "AT+SAPBR=3,1,\"Contype\",\"GPRS\"\r",
            client.sent_commands.get(5).unwrap()
        );
        assert_eq!("AT+SAPBR=1,1\r", client.sent_commands.get(6).unwrap());
        assert_eq!(
            "AT+CLBSCFG=1,3,\"lbs-simcom.com:3002\"\r",
            client.sent_commands.get(7).unwrap()
        );
        assert_eq!("AT+CLBS=4,1\r", client.sent_commands.get(8).unwrap());
        assert_eq!("AT+CLBS=4,1\r", client.sent_commands.get(9).unwrap());
        assert_eq!("AT+CLBS=4,1\r", client.sent_commands.get(10).unwrap());
        assert_eq!("AT+SAPBR=0,1\r", client.sent_commands.get(11).unwrap());
        assert_eq!("AT+CGATT=0\r", client.sent_commands.get(12).unwrap());
        assert_eq!(
            location::Location {
                latitude: 46.7624859,
                longitude: 18.6304591,
                accuracy: 550.0,
                timestamp: 1670846541000,
            },
            loc1.unwrap()
        );
        assert_eq!(2, pico.sleep_calls.len());
        assert_eq!(1000, *pico.sleep_calls.get(0).unwrap());
        assert_eq!(1000, *pico.sleep_calls.get(1).unwrap());
    }

    // TODO test error handling
}
