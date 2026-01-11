use defmt::Format;

use alloc::format;
use alloc::string::ToString;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless::String;
use defmt::info;

use crate::at::NoResponse;
use crate::utils::send_command_logged;

// 4.2.2 AT+CMGF Select SMS Message Format
// AT+CMGF=[<mode>]
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CMGF", NoResponse)]
pub struct AtSelectSMSMessageFormatWrite {
    pub mode: MessageMode,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum MessageMode {
    PDU = 0,
    Text = 1,
}

// 4.2.5 AT+CMGS Send SMS Message
// AT+CMGS=<da>[,[<toda>]]<CR>text is entered[ctrl-Z/ESC]
#[derive(Clone, Debug)]
pub struct AtSendSMSWrite {
    pub number: String<16>,
    pub message: String<160>,
}

impl<'a> AtatCmd for AtSendSMSWrite {
    type Response = NoResponse;

    const MAX_LEN: usize = 16;

    // TODO this is not working!
    fn write(&self, buf: &mut [u8]) -> usize {
        let formatted = format!("AT+CMGS={}\r{}\x1a", self.number, self.message);
        let cmd = formatted.as_bytes();
        let len = cmd.len();
        buf[..len].copy_from_slice(cmd);
        len
    }

    fn parse(&self, _: Result<&[u8], atat::InternalError>) -> Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }
}

// 4.2.3 AT+CMGL List SMS Messages from Preferred Store
// AT+CMGL=<stat>[,<mode>]
// Not Implemented, Read SMS one by one will be enough for this project

// 4.2.4 AT+CMGR Read SMS Message
// AT+CMGR=<index>[,<mode>]
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CMGR", SMSMessageResponse, timeout_ms = 5000)]
pub struct AtReadSMSMessagesWrite {
    pub index: u32,
    pub mode: Option<ReadSMSMode>,
}

#[derive(Debug, Default, Format, Clone, PartialEq, AtatEnum)]
pub enum ReadSMSMode {
    #[default]
    Normal = 0,
    NotChangeStatusOfSMSRecord = 1,
}

// for CBM storage:
// +CMGR: <stat>,<sn>,<mid>,<dcs>,<page>,<pages><CR><LF><data>
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
pub struct SMSMessageResponse {
    stat: String<30>,
    sn: String<30>,
    mid: Option<String<30>>,
    date_time: String<30>,
    message: String<256>,
}

// 4.2.8 AT+CNMI New SMS Message Indications
// AT+CNMI=<mode>[,<mt>[,<bm>[,<ds>[,<bfr>]]]]
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CNMI", NoResponse)]
pub struct AtNewSMSMessageIndicationsWrite {
    pub mode: SMSIndicationsMode,
    pub mt: Option<MtMode>,
    pub bm: Option<CBMMode>,
    pub ds: Option<DSMode>,
    pub bfr: Option<BFRMode>,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum SMSIndicationsMode {
    BufferUCRinTA = 0,
    DiscardIndicationRejectNewUCR = 1,
    BufferUCRInTA = 2,
    ForwardUCRDirectlyToTE = 3,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum MtMode {
    NoSMSDeliverIndicationsToTE = 0,
    SMSStoredInMETAToTE = 1, // +CMTI: <mem>,<index>
    SMSDirectlyToTE = 2,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum CBMMode {
    NoCBMToTE = 0,
    CBMDirectlyToTE = 2,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum DSMode {
    NoDSToTE = 0,
    DSirectlyToTE = 1,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum BFRMode {
    TAFlushedToTE = 0,
    TACleared = 1,
}
// +CMTI: <mem3>,<index>
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
pub struct NewMessageIndicationUrc {
    pub mem: String<30>,
    pub index: i32,
}

pub async fn init<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,
) {
    send_command_logged(
        client,
        &AtSelectSMSMessageFormatWrite {
            mode: MessageMode::Text,
        },
        "AtSelectSMSMessageFormatWrite".to_string(),
    )
    .await
    .ok();

    send_command_logged(
        client,
        &AtNewSMSMessageIndicationsWrite {
            mode: SMSIndicationsMode::BufferUCRInTA,
            mt: Some(MtMode::SMSStoredInMETAToTE),
            bm: Some(CBMMode::NoCBMToTE),
            ds: Some(DSMode::NoDSToTE),
            bfr: Some(BFRMode::TAFlushedToTE),
        },
        "AtNewSMSMessageIndicationsWrite".to_string(),
    )
    .await
    .ok();
}

// TODO this is not working yet, need to debug
pub async fn send_sms<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,         // TODO:
    number: &'static str,  // Bytes<16>  ? (same for Call)
    message: &'static str, // Bytes<160> ?
) {
    send_command_logged(
        client,
        &AtSendSMSWrite {
            number: String::<16>::try_from(number).unwrap(),
            message: String::<160>::try_from(message).unwrap(),
        },
        "AtSendSMSWrite".to_string(),
    )
    .await
    .ok();
}

// todo, temporary helper
pub async fn receive_sms<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,
) {
    for i in 1..1000 {
        match send_command_logged(
            client,
            &AtReadSMSMessagesWrite {
                index: i,
                mode: None,
            },
            "AtReadSMSMessagesWrite".to_string(),
        )
        .await
        {
            Ok(v) => {
                info!(
                    "SMS RESP state={} date={} sender={} message={}",
                    v.stat, v.date_time, v.sn, v.message
                );
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd_serialization_tests;

    use super::*;
    use atat::AtatCmd;

    cmd_serialization_tests! {
        test_at_select_sms_message_format_write: (
            AtSelectSMSMessageFormatWrite {
                mode: MessageMode::Text,
            },
            "AT+CMGF=1\r",
        ),
        test_at_send_sms_write: (
            AtSendSMSWrite {
                number: String::try_from("+361234567").unwrap(),
                message: String::try_from("this is the message content").unwrap(),
            },
            "AT+CMGS=+361234567\rthis is the message content\u{1a}",
        ),
        test_at_new_sms_message_indications_write: (
            AtNewSMSMessageIndicationsWrite {
                mode: SMSIndicationsMode::BufferUCRInTA,
                mt: Some(MtMode::SMSStoredInMETAToTE),
                bm: Some(CBMMode::NoCBMToTE),
                ds: Some(DSMode::NoDSToTE),
                bfr: Some(BFRMode::TAFlushedToTE),
            },
            "AT+CNMI=2,1,0,0,0\r",
        ),
        test_at_read_sms_messages_write: (
            AtReadSMSMessagesWrite {
                index: 42,
                mode: None,
            },
            "AT+CMGR=42\r",
        ),
    }

    #[test]
    fn test_clip_response() {
        #[derive(Clone, Debug, Format, AtatCmd)]
        #[at_cmd("+CMTI", NewMessageIndicationUrc, timeout_ms = 15000)]
        struct AtUrcHack;

        let cmd = AtUrcHack;
        assert_eq!(
            NewMessageIndicationUrc {
                mem: String::try_from("SM").unwrap(),
                index: 1
            },
            cmd.parse(Ok(b"+CMTI: \"SM\",1\r\n")).unwrap()
        );
    }

    #[tokio::test]
    async fn test_sms_init() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        init(&mut client, &mut pico).await;
        assert_eq!(2, client.sent_commands.len());
        assert_eq!("AT+CMGF=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("AT+CNMI=2,1,0,0,0\r", client.sent_commands.get(1).unwrap());
    }

    #[tokio::test]
    async fn test_send_sms() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok(">".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        send_sms(
            &mut client,
            &mut pico,
            "+36301234567",
            "this is the text message",
        )
        .await;
        assert_eq!(1, client.sent_commands.len());
        assert_eq!(
            "AT+CMGS=+36301234567\rthis is the text message\u{1a}",
            client.sent_commands.get(0).unwrap()
        );
    }

    #[tokio::test]
    async fn test_receive_sms() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("+CMGR: \"REC READ\",\"+36301234567\",\"\",\"26/01/10,17:25:32+04\"\r\n$tATA/location/12345".as_bytes()));
        client.results.push_back(Err(atat::InternalError::Timeout));

        let mut pico = crate::at::tests::PicoMock::default();
        receive_sms(&mut client, &mut pico).await;
        assert_eq!(2, client.sent_commands.len());
        assert_eq!("AT+CMGR=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("AT+CMGR=2\r", client.sent_commands.get(1).unwrap());
    }
}
