use atat::atat_derive::AtatResp;
use atat::atat_derive::AtatUrc;

#[derive(Clone, AtatResp)]
pub struct MessageWaitingIndication;

#[derive(Clone, AtatUrc)]
pub enum Urc {
    #[at_urc("+UMWI")]
    MessageWaitingIndication(super::urc::MessageWaitingIndication),
}
