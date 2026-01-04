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
    use super::*;
    use atat::AtatCmd;
    use atat::heapless::Vec;

    fn zeros() -> Vec<u8, 127> {
        let mut buffer = Vec::<u8, 127>::new();
        for _ in 0..127 {
            let _ = buffer.push(0u8);
        }
        return buffer;
    }

    #[test]
    fn test_at_init() {
        let cmd = AtInit {};
        let mut buffer = zeros();
        assert_eq!(3, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT\r"
        );
    }

    #[test]
    fn test_set_command_echo_off() {
        let cmd = AtSetCommandEchoOff;
        let mut buffer = zeros();
        assert_eq!(5, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "ATE0\r"
        );
    }

    #[test]
    fn test_network_registration() {
        let cmd = AtNetworkRegistrationRead;
        let mut buffer = zeros();
        assert_eq!(10, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+CGREG?\r"
        );

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
    fn test_enter_pin() {
        let cmd = AtEnterPinRead;
        let mut buffer = zeros();
        assert_eq!(9, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+CPIN?\r"
        );

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
    fn test_signal_quality_report_execute() {
        let cmd = AtSignalQualityReportExecute;
        let mut buffer = zeros();
        assert_eq!(7, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+CSQ\r"
        );

        assert_eq!(
            SignalQualityReportResponse { rssi: 19, ber: 0 },
            cmd.parse(Ok(b"+CSQ: 19,0\r\n")).unwrap()
        );
    }

    #[test]
    fn test_operator_selection_read() {
        let cmd = AtOperatorSelectionRead;
        let mut buffer = zeros();
        assert_eq!(9, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+COPS?\r"
        );

        assert_eq!(
            OperatorSelectionReadResponse {
                mode: 0,
                format: Some(0),
                oper: Some(String::try_from("PANNON GSM").unwrap())
            },
            cmd.parse(Ok(b"+CSQ: 0,0,\"PANNON GSM\"\r\n")).unwrap()
        );
    }

    #[test]
    fn test_at_enable_or_disable_initial_urc_presentation_write() {
        let cmd = AtEnableOrDisableInitialURCPresentationWite { mode: 0 };
        let mut buffer = zeros();
        assert_eq!(11, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+CIURC=0\r"
        );
    }
}
