use alloc::format;
use alloc::string::ToString;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::heapless::String;

use crate::at::NoResponse;
use crate::utils::send_command_logged;

// 4.2.2 AT+CMGF Select SMS Message Format
// AT+CMGF=[<mode>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CMGF", NoResponse)]
pub struct AtSelectSMSMessageFormatWrite {
    pub mode: MessageMode,
}

#[derive(Debug, Clone, PartialEq, AtatEnum)]
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

pub async fn send_sms<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,         // TODO:
    number: &'static str,  // Bytes<16>  ? (same for Call)
    message: &'static str, // Bytes<160> ?
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
        &AtSendSMSWrite {
            number: String::<16>::try_from(number).unwrap(),
            message: String::<160>::try_from(message).unwrap(),
        },
        "AtSendSMSWrite".to_string(),
    )
    .await
    .ok();
}

#[cfg(test)]
mod tests {
    use crate::{at, cmd_serialization_tests};

    use super::*;
    use atat::AtatCmd;

    cmd_serialization_tests! {
        test_at_select_sms_message_format_write: (
            AtSelectSMSMessageFormatWrite {
                mode: MessageMode::Text,
            },
            10,
            "AT+CMGF=1\r",
        ),
        test_at_send_sms_write: (
            AtSendSMSWrite {
                number: String::try_from("+361234567").unwrap(),
                message: String::try_from("this is the message content").unwrap(),
            },
            47,
            "AT+CMGS=+361234567\rthis is the message content\u{1a}",
        ),
    }

    #[tokio::test]
    async fn test_send_sms() {
        at::tests::init_env_logger();

        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok(">".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        send_sms(
            &mut client,
            &mut pico,
            "+36301234567",
            "this is the text message",
        )
        .await;
        assert_eq!(2, client.sent_commands.len());
        assert_eq!("AT+CMGF=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!(
            "AT+CMGS=+36301234567\rthis is the text message\u{1a}",
            client.sent_commands.get(1).unwrap()
        );
    }
}
