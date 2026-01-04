use atat::atat_derive::AtatResp;
use atat::atat_derive::AtatUrc;

// 18.1 CME ERROR
// +CME ERROR: <err>
#[derive(Clone, AtatResp)]
pub struct CmeError {
    pub err: u16,
}

// 18.2 CMS ERROR
// +CMS ERROR: <err>
#[derive(Clone, AtatResp)]
pub struct CmsError {
    pub err: u16,
}

// 18.3 Summary of Unsolicited Result Codes

#[derive(Clone, AtatUrc)]
pub enum Urc {
    #[at_urc("+CME ERROR")]
    CmeError(super::urc::CmeError),
    #[at_urc("+CMS ERROR")]
    CmsError(super::urc::CmsError),
    #[at_urc("Call Ready")]
    CallReady,
    #[at_urc("SMS Ready")]
    SMSReady,
}
