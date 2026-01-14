use defmt::Format;

use alloc::string::ToString;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless::String;
use defmt::info;

use crate::at::NoResponse;
use crate::hexstr::UCS2HexString;
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
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CMGS", SMSMessageResponse, timeout_ms = 5000)]
pub struct AtSMSSend {
    pub number: String<30>,
}
#[derive(Clone, Debug)]
pub struct AtSMSData {
    pub message: String<160>,
}

impl<'a> AtatCmd for AtSMSData {
    type Response = SMSDataResponse;

    const MAX_LEN: usize = 160;
    const MAX_TIMEOUT_MS: u32 = 60000;

    fn write(&self, buf: &mut [u8]) -> usize {
        let bytes = self.message.as_bytes();
        let len = bytes.len();
        let ctrl_z = b"\x1a";
        buf[..len].copy_from_slice(bytes);
        buf[len..len + ctrl_z.len()].copy_from_slice(ctrl_z);
        len + ctrl_z.len()
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        match resp {
            Ok(v) => {
                let s = core::str::from_utf8(&v["+CMGS: ".len()..])
                    .map_err(|_o| -> atat::Error { atat::Error::Parse })?;
                let mr: i32 = s
                    .parse()
                    .map_err(|_o| -> atat::Error { atat::Error::Parse })?;
                return Ok(SMSDataResponse { mr: mr });
            }
            Err(_) => Err(atat::Error::Parse),
        }
    }
}

// +CMGS: <mr>
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
pub struct SMSDataResponse {
    mr: i32, // GSM 03.40 TP-Message-Reference in integer format
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
    sn: UCS2HexString<64>,
    mid: Option<String<30>>,
    date_time: String<30>,
    // Text mode + UCS2 charset is assumed
    message: UCS2HexString<1024>,
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

// 3.2.12 AT+CSCS Select TE Character Set
// AT+CSCS=<chset>
// The character set affects transmission and reception of SMS and SMS Cell Broadcast messages,
// the entry and display of phone book entries text field and SIM Application Toolkit alpha strings.
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CSCS", NoResponse)]
pub struct AtSelectTECharsetWrite {
    pub chset: String<30>, // "GSM" 7-bit, "UCS2", "IRA", "HEX", "PCCP", "PCDN", "8859-1"
}

pub async fn init<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,
) {
    // PDU mode might make more sense, currently HEX + Text mod is assumed.
    // http://rfc.nop.hu/sms/default.htm
    // https://en.wikipedia.org/wiki/GSM_03.40
    send_command_logged(
        client,
        &AtSelectSMSMessageFormatWrite {
            mode: MessageMode::Text,
        },
        "AtSelectSMSMessageFormatWrite".to_string(),
    )
    .await
    .ok();

    // TODO: SMSSending is not adjusted yet. Either use plain text for send and UCS2 only for receive or try it with UCS2HexString<> parameters.
    send_command_logged(
        client,
        &AtSelectTECharsetWrite {
            chset: String::try_from("UCS2").unwrap(),
        },
        "AtSelectTECharsetWrite".to_string(),
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

pub async fn send_sms<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,
    number: &String<30>,
    message: &String<160>,
) {
    send_command_logged(
        client,
        &AtSMSSend {
            number: number.clone(),
        },
        "AtSMSSend".to_string(),
    )
    .await
    .ok();
    send_command_logged(
        client,
        &AtSMSData {
            message: message.clone(),
        },
        "AtSMSData".to_string(),
    )
    .await
    .ok();
}

// TODO, this is just a temporary helper function.
pub async fn receive_sms<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    pico: &mut U,
) {
    for i in 1..100 {
        pico.sleep(100).await;
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
        test_at_send_sms_1: (
            AtSMSSend {
                number: String::try_from("+361234567").unwrap(),
            },
            "AT+CMGS=\"+361234567\"\r",
        ),
        test_at_send_sms_2: (
            AtSMSData {
                message: String::try_from("this is the message").unwrap(),
            },
            "this is the message\x1a",
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

    #[test]
    fn test_send_sms_response() {
        let cmd = AtSMSData {
            message: String::try_from("data").unwrap(),
        };

        assert_eq!(
            SMSDataResponse { mr: 13 },
            cmd.parse(Ok(b"+CMGS: 13")).unwrap(),
        );
    }

    #[test]
    fn test_sms_response() {
        let cmd = AtReadSMSMessagesWrite {
            index: 0,
            mode: None,
        };

        assert_eq!(
            SMSMessageResponse {
                stat: String::try_from("REC READ").unwrap(),
                sn: UCS2HexString { text: String::try_from("+36301234567").unwrap(), quoted: true },
                mid: Some(String::new()),
                date_time: String::try_from("25/04/25,10:37:39+08").unwrap(),
                message: UCS2HexString { text: String::try_from("$tATA/location/12345").unwrap(), quoted: false },
            },
            cmd.parse(Ok(b"+CMGR: \"REC READ\",\"002B00330036003300300031003200330034003500360037\",\"\",\"25/04/25,10:37:39+08\"\r\n00240074004100540041002F006C006F0063006100740069006F006E002F00310032003300340035\r\n"))
                .unwrap(),
        );

        assert_eq!(
            SMSMessageResponse {
                stat: String::try_from("REC READ").unwrap(),
                sn: UCS2HexString { text: String::try_from("+36301234567").unwrap(), quoted: true },
                mid: Some(String::new()),
                date_time: String::try_from("25/04/25,10:37:39+08").unwrap(),
                message: UCS2HexString { text: String::try_from("Köszönjük.\nÜdvözlettel Yettel.").unwrap(), quoted: false },
            },
            cmd.parse(Ok(b"+CMGR: \"REC READ\",\"002B00330036003300300031003200330034003500360037\",\"\",\"25/04/25,10:37:39+08\"\r\n004B00F60073007A00F6006E006A00FC006B002E000A00DC0064007600F6007A006C0065007400740065006C002000590065007400740065006C002E\r\n"))
                .unwrap(),
        );

        assert_eq!(
            SMSMessageResponse {
                stat: String::try_from("REC READ").unwrap(),
                sn: UCS2HexString { text: String::try_from("+36301234567").unwrap(), quoted: true },
                mid: Some(String::new()),
                date_time: String::try_from("25/04/25,10:37:39+08").unwrap(),
                message: UCS2HexString { text: String::try_from("Tamás Dömők 😎").unwrap(), quoted: false },
            },
            cmd.parse(Ok(b"+CMGR: \"REC READ\",\"002B00330036003300300031003200330034003500360037\",\"\",\"25/04/25,10:37:39+08\"\r\n00540061006D00E100730020004400F6006D0151006B0020D83DDE0E\r\n"))
                .unwrap(),
        );
    }

    #[tokio::test]
    async fn test_sms_init() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        init(&mut client, &mut pico).await;
        assert_eq!(3, client.sent_commands.len());
        assert_eq!("AT+CMGF=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("AT+CSCS=\"UCS2\"\r", client.sent_commands.get(1).unwrap());
        assert_eq!("AT+CNMI=2,1,0,0,0\r", client.sent_commands.get(2).unwrap());
    }

    #[tokio::test]
    async fn test_send_sms() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok(">".as_bytes()));
        client.results.push_back(Ok("+CMGS: 1".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        send_sms(
            &mut client,
            &mut pico,
            &String::try_from("+36301234567").unwrap(),
            &String::try_from("this is the text message").unwrap(),
        )
        .await;
        assert_eq!(2, client.sent_commands.len());
        assert_eq!(
            "AT+CMGS=\"+36301234567\"\r",
            client.sent_commands.get(0).unwrap()
        );
        assert_eq!(
            "this is the text message\x1a",
            client.sent_commands.get(1).unwrap()
        );
    }

    #[tokio::test]
    async fn test_receive_sms() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("+CMGR: \"REC READ\",\"002B00330036003300300031003200330034003500360037\",\"\",\"26/01/10,17:25:32+04\"\r\n00240074004100540041002F006C006F0063006100740069006F006E002F00310032003300340035".as_bytes()));
        client.results.push_back(Err(atat::InternalError::Timeout));

        let mut pico = crate::at::tests::PicoMock::default();
        receive_sms(&mut client, &mut pico).await;
        assert_eq!(2, client.sent_commands.len());
        assert_eq!("AT+CMGR=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("AT+CMGR=2\r", client.sent_commands.get(1).unwrap());
    }
}
