use atat::atat_derive::AtatResp;
use atat::atat_derive::AtatUrc;

// 18.1 CME ERROR

// 18.2 CMS ERROR

// 18.3 Summary of Unsolicited Result Codes

#[derive(Clone, AtatResp)]
pub struct MessageWaitingIndication;

#[derive(Clone, AtatUrc)]
pub enum Urc {
    #[at_urc("+UMWI")]
    MessageWaitingIndication(super::urc::MessageWaitingIndication),
}
