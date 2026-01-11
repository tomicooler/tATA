use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use defmt::Format;

// 3.2.52 AT+CBC Battery Charge
// AT+CBC
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CBC", BatteryChargeResponse)]
pub struct AtBatteryChargeExecute;

// +CBC: <bcs>,<bcl>,<voltage>
#[derive(Debug, Format, Clone, AtatResp, PartialEq)]
pub struct BatteryChargeResponse {
    #[at_arg(position = 0)]
    pub bcs: BatteryStatus,
    #[at_arg(position = 1)]
    pub bcl: u8,
    #[at_arg(position = 2)]
    pub voltage: u32,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum BatteryStatus {
    NotCharging = 0,
    Charging = 1,
    ChargingFinished = 2,
}

#[cfg(test)]
mod tests {
    use crate::cmd_serialization_tests;

    use super::*;
    use atat::AtatCmd;

    cmd_serialization_tests! {
        test_at_battery_charge_execute: (
            AtBatteryChargeExecute,
            "AT+CBC\r",
        ),
    }

    #[test]
    fn test_network_registration_responses() {
        let cmd = AtBatteryChargeExecute;

        assert_eq!(
            BatteryChargeResponse {
                bcs: BatteryStatus::NotCharging,
                bcl: 50,
                voltage: 300,
            },
            cmd.parse(Ok(b"+CBC: 0,50,300\r\n")).unwrap()
        );

        assert_eq!(
            BatteryChargeResponse {
                bcs: BatteryStatus::Charging,
                bcl: 75,
                voltage: 450,
            },
            cmd.parse(Ok(b"+CBC: 1,75,450\r\n")).unwrap()
        );

        assert_eq!(
            BatteryChargeResponse {
                bcs: BatteryStatus::ChargingFinished,
                bcl: 100,
                voltage: 600,
            },
            cmd.parse(Ok(b"+CBC: 2,100,600\r\n")).unwrap()
        );
    }
}
