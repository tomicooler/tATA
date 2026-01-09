use atat::atat_derive::AtatResp;
use atat::atat_derive::AtatUrc;
use atat::heapless_bytes::Bytes;

use crate::call::ClipUrc;
use crate::sms::NewMessageIndicationUrc;

// 18.1 CME ERROR
// +CME ERROR: <err>
// 18.2 CMS ERROR
// +CMS ERROR: <err>
// These are handled by the atat library

// DEACT
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
pub struct DeactResponse {
    pub deact: Bytes<5>,
}

// 18.3 Summary of Unsolicited Result Codes
// All URCs must be defined (https://github.com/FactbirdHQ/atat/issues/149#issuecomment-1538193692)
#[derive(Clone, AtatUrc)]
pub enum Urc {
    #[at_urc("RING")]
    Ring,
    #[at_urc("NORMAL POWER DOWN")]
    NormalPowerDown,
    #[at_urc("UNDER-VOLTAGE POWER DOWN")]
    UnderVoltagePowerDown,
    #[at_urc("UNDER-VOLTAGE WARNING")]
    UnderVoltageWarning,
    #[at_urc("OVER-VOLTAGE POWER DOWN")]
    OverVoltagePowerDown,
    #[at_urc("OVER-VOLTAGE WARNING")]
    OverVoltageWarning,
    #[at_urc("CHARGE-ONLY MODE")]
    ChargeOnlyMode,
    #[at_urc("RDY")]
    Ready,
    #[at_urc("Call Ready")]
    CallReady,
    #[at_urc("SMS Ready")]
    SMSReady,
    #[at_urc("1 CONNECT OK")]
    ConnectOK1,
    #[at_urc("CONNECT OK")]
    ConnectOK,
    #[at_urc("+SAPBR 1")] // 1 is the connection id +SAPBR <cid>: DEACT
    SetBearer(DeactResponse),
    #[at_urc("+PDP")]
    GprsDisconnected(DeactResponse),
    #[at_urc("+CLIP")]
    ClipUrc(ClipUrc),
    #[at_urc("+CMTI")]
    NewMessageIndicationUrc(NewMessageIndicationUrc),
}
