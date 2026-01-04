use atat::atat_derive::AtatUrc;

// 18.1 CME ERROR
// +CME ERROR: <err>
// 18.2 CMS ERROR
// +CMS ERROR: <err>
// These are handled by the atat library

// 18.3 Summary of Unsolicited Result Codes

#[derive(Clone, AtatUrc)]
pub enum Urc {
    #[at_urc("Call Ready")]
    CallReady,
    #[at_urc("SMS Ready")]
    SMSReady,
}
