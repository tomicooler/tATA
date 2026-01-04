use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless::String;

use crate::at::NoResponse;

#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("AT", NoResponse, cmd_prefix = "", timeout_ms = 5000)]
pub struct AtInit;

// 2.2.7 ATE Set Command Echo Mode
//  ATE1 - Echo On
//  ATE0 - Echo Off
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ATE0", NoResponse, cmd_prefix = "")]
pub struct AtSetCommandEchoOff;

// 3.2.32 AT+CGREG Network Registration
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CGREG?", NetworkRegistrationReadResponse)]
pub struct AtNetworkRegistrationRead;

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum NetworkRegistrationStatus {
    Registered = 1,
    Searching = 2,
    Denied = 3,
    Unknown = 4,
    RegisteredRoaming = 5,
}

// +CGREG: <n>,<stat>[,[lac],[ci]]
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct NetworkRegistrationReadResponse {
    #[at_arg(position = 0)]
    pub n: u8,
    #[at_arg(position = 1)]
    pub stat: NetworkRegistrationStatus,
    #[at_arg(position = 2)]
    pub lac: Option<String<4>>,
    pub ci: Option<String<4>>,
}

// 3.2.28 AT+CPIN Enter Pin
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CPIN?", EnterPinReadResponse, timeout_ms = 5000)]
pub struct AtEnterPinRead;

// +CPIN: <code>
//        READY, SIM PIN, SIM PUK, PH_SIM PIN, PH_SIM PUK, SIM PIN2, SIM PUK2
// +CPIN: NOT READY
// +CPIN: NOT INSERTED
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct EnterPinReadResponse {
    #[at_arg(position = 0)]
    pub code: String<16>,
}

// 3.2.53 AT+CSQ Signal Quality Report
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CSQ", SignalQualityReportResponse)]
pub struct AtSignalQualityReportExecute;

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct SignalQualityReportResponse {
    #[at_arg(position = 0)]
    pub rssi: u8, // 0: -115 dBm or less, 1: -111 dBm, 2..30: -110...-54 dBm, 31: -52 dBm or greater, 99: not known
    #[at_arg(position = 1)]
    pub ber: u8,
}

// 3.2.22 AT+COPS Operator Selection
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+COPS?", OperatorSelectionReadResponse)]
pub struct AtOperatorSelectionRead;

// +COPS: <mode>[,<format>,<oper>]
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OperatorSelectionReadResponse {
    #[at_arg(position = 0)]
    pub mode: u8, // 0 Automatic, 1 Manual
    #[at_arg(position = 1)]
    pub format: Option<u8>,
    pub oper: Option<String<64>>,
}

// 6.2.33 AT+CIURC Enable or Disable Initial URC Presentation
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CIURC", NoResponse)]
pub struct AtEnableOrDisableInitialURCPresentationWite {
    pub mode: u8,
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use crate::cmd_serialization_tests;

    use super::*;
    use atat::AtatCmd;

    cmd_serialization_tests! {
        test_at_init: (
            AtInit,
            3,
            "AT\r",
        ),
        test_set_command_echo_off: (
            AtSetCommandEchoOff,
            5,
            "ATE0\r",
        ),
        test_network_registration: (
            AtNetworkRegistrationRead,
            10,
            "AT+CGREG?\r",
        ),
        test_enter_pin: (
            AtEnterPinRead,
            9,
            "AT+CPIN?\r",
        ),
        test_signal_quality_report_execute: (
            AtSignalQualityReportExecute,
            7,
            "AT+CSQ\r",
        ),
        test_operator_selection_read: (
            AtOperatorSelectionRead,
            9,
            "AT+COPS?\r",
        ),
        test_at_enable_or_disable_initial_urc_presentation_write: (
            AtEnableOrDisableInitialURCPresentationWite { mode: 0 },
            11,
            "AT+CIURC=0\r",
        ),
    }

    #[test]
    fn test_network_registration_responses() {
        let cmd = AtNetworkRegistrationRead;

        assert_eq!(
            NetworkRegistrationReadResponse {
                n: 0,
                stat: NetworkRegistrationStatus::Searching,
                lac: None,
                ci: None,
            },
            cmd.parse(Ok(b"+CGREG: 0,2\r\n")).unwrap()
        );

        assert_eq!(
            NetworkRegistrationReadResponse {
                n: 0,
                stat: NetworkRegistrationStatus::Registered,
                lac: None,
                ci: None,
            },
            cmd.parse(Ok(b"+CGREG: 0,1\r\n")).unwrap()
        );

        assert_eq!(
            NetworkRegistrationReadResponse {
                n: 0,
                stat: NetworkRegistrationStatus::Denied,
                lac: None,
                ci: None,
            },
            cmd.parse(Ok(b"+CGREG: 0,3\r\n")).unwrap()
        );

        assert_eq!(
            NetworkRegistrationReadResponse {
                n: 0,
                stat: NetworkRegistrationStatus::Unknown,
                lac: None,
                ci: None,
            },
            cmd.parse(Ok(b"+CGREG: 0,4\r\n")).unwrap()
        );

        assert_eq!(
            NetworkRegistrationReadResponse {
                n: 2,
                stat: NetworkRegistrationStatus::RegisteredRoaming,
                lac: Some(String::try_from("00").unwrap()),
                ci: Some(String::try_from("FF").unwrap()),
            },
            cmd.parse(Ok(b"+CGREG: 2,5,\"00\",\"FF\"\r\n")).unwrap()
        );
    }

    #[test]
    fn test_enter_pin_responses() {
        let cmd = AtEnterPinRead;

        assert_eq!(
            EnterPinReadResponse {
                code: String::try_from("READY").unwrap()
            },
            cmd.parse(Ok(b"+CPIN: READY\r\n")).unwrap()
        );
        assert_eq!(
            EnterPinReadResponse {
                code: String::try_from("NOT READY").unwrap()
            },
            cmd.parse(Ok(b"+CPIN: NOT READY\r\n")).unwrap()
        );
    }

    #[test]
    fn test_signal_quality_report_execute_responses() {
        let cmd = AtSignalQualityReportExecute;
        assert_eq!(
            SignalQualityReportResponse { rssi: 19, ber: 0 },
            cmd.parse(Ok(b"+CSQ: 19,0\r\n")).unwrap()
        );
    }

    #[test]
    fn test_operator_selection_read_responses() {
        let cmd = AtOperatorSelectionRead;
        assert_eq!(
            OperatorSelectionReadResponse {
                mode: 0,
                format: Some(0),
                oper: Some(String::try_from("PANNON GSM").unwrap())
            },
            cmd.parse(Ok(b"+CSQ: 0,0,\"PANNON GSM\"\r\n")).unwrap()
        );
    }
}
