use alloc::format;
use alloc::string::ToString;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::atat_derive::AtatResp;
use atat::heapless::String;
use defmt::Format;

use crate::at::NoResponse;
use crate::utils::LogBE;
use crate::utils::send_command_logged;

// 6.2.19 AT+CHFA Swap the Audio Channels
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CHFA", NoResponse)]
pub struct AtSwapAudioChannelsWrite {
    pub n: AudioChannels,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum AudioChannels {
    Main = 1,
    Aux = 2,
    MainHandFree = 3,
    AuxHandFree = 4,
    PCM = 5,
}

// 2.2.3 ATD Mobile Originated Call to Dial A Number
// ATD<n>[<mgsm>][;]
// ATD+36301234567,i;   <- call this number and i: Deactivates CLIR (Enable presentation of own number to called party)
#[derive(Clone, Debug)]
pub struct AtDialNumber {
    pub number: String<30>,
}

impl<'a> AtatCmd for AtDialNumber {
    type Response = NoResponse;

    const MAX_LEN: usize = 16;

    fn write(&self, buf: &mut [u8]) -> usize {
        let formatted = format!("ATD{},i;\r", self.number);
        let cmd = formatted.as_bytes();
        let len = cmd.len();
        buf[..len].copy_from_slice(cmd);
        len
    }

    fn parse(&self, _: Result<&[u8], atat::InternalError>) -> Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }
}

// AT+CHUP; hang up the call
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CHUP;", NoResponse)]
pub struct AtHangup;

// 3.2.18 AT+CLIP Calling Line Identification Presentation
// AT+CLIP=<n>
#[derive(Clone, Debug, Format, AtatCmd)]
#[at_cmd("+CLIP", NoResponse, timeout_ms = 15000)]
pub struct AtCallingLineIdentificationPresentationWrite {
    pub n: ClipMode,
}

#[derive(Debug, Format, Clone, PartialEq, AtatEnum)]
pub enum ClipMode {
    DisableClipNotification = 0,
    EnableClipNotification = 1, // +CLIP URC
}

// <number>,<type>[,<subaddr>,<satype>,<alphaId>,<CLI validity>]
#[derive(Debug, Clone, AtatResp, PartialEq, Default)]
pub struct ClipUrc {
    pub number: String<30>,
    pub type_: ClipType,
    pub sub_addr: Option<String<30>>,
    pub sa_type: Option<i32>,
    pub alpha_id: Option<String<30>>,
    pub cli_validity: Option<ClipValidity>,
}

#[derive(Debug, Default, Format, Clone, PartialEq, AtatEnum)]
pub enum ClipType {
    #[default]
    Unknown = 129,
    National = 161,
    International = 145,
    NetworkSpecific = 177,
}

#[derive(Debug, Default, Format, Clone, PartialEq, AtatEnum)]
pub enum ClipValidity {
    #[default]
    Valid = 0,
    Withheld = 1,
    NotAvailable = 2,
}

pub async fn init<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    _pico: &mut U,
) {
    // Note, this is NO_SAVE, and by default is enabled on my device.
    // It takes a bit of time, but having an extra Read command messes
    // up the URC handling.
    send_command_logged(
        client,
        &AtCallingLineIdentificationPresentationWrite {
            n: ClipMode::EnableClipNotification,
        },
        "AtCallingLineIdentificationPresentationWrite".to_string(),
    )
    .await
    .ok();
}

pub async fn call_number<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    pico: &mut U,
    number: &String<30>,
    duration_millis: u64,
) {
    send_command_logged(
        client,
        &AtSwapAudioChannelsWrite {
            n: AudioChannels::Main,
        },
        "AtSwapAudioChannelsWrite".to_string(),
    )
    .await
    .ok();

    send_command_logged(
        client,
        &AtDialNumber {
            number: number.clone(),
        },
        "AtSwapAudioChannelsAtDialNumberWrite".to_string(),
    )
    .await
    .ok();

    {
        let _l = LogBE::new("Sleeping".to_string());
        pico.sleep(duration_millis).await;
    }

    send_command_logged(client, &AtHangup, "AtHangup".to_string())
        .await
        .ok();
}

#[cfg(test)]
mod tests {
    use crate::cmd_serialization_tests;

    use super::*;
    use atat::AtatCmd;

    cmd_serialization_tests! {
        test_at_swap_audio_channels_write: (
            AtSwapAudioChannelsWrite {
                n: AudioChannels::Main,
            },
            "AT+CHFA=1\r",
        ),
        test_at_dial_number: (
            AtDialNumber {
                number: String::try_from("+361234567").unwrap(),
            },
            "ATD+361234567,i;\r",
        ),
        test_at_hangup: (
            AtHangup,
            "AT+CHUP;\r",
        ),
        test_at_calling_line_identification_presentation_write: (
            AtCallingLineIdentificationPresentationWrite {
                n: ClipMode::EnableClipNotification,
            },
            "AT+CLIP=1\r",
        ),
    }

    #[test]
    fn test_clip_response() {
        #[derive(Clone, Debug, Format, AtatCmd)]
        #[at_cmd("+CLIP", ClipUrc, timeout_ms = 15000)]
        struct AtUrcHack;

        let cmd = AtUrcHack;
        assert_eq!(
            ClipUrc {
                number: String::try_from("+36301234567").unwrap(),
                type_: ClipType::International,
                sub_addr: Some(String::new()),
                sa_type: Some(0),
                alpha_id: Some(String::new()),
                cli_validity: Some(ClipValidity::Valid),
            },
            cmd.parse(Ok(b"+CLIP: \"+36301234567\",145,\"\",0,\"\",0\r\n"))
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_sms_init() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        init(&mut client, &mut pico).await;
        assert_eq!(1, client.sent_commands.len());
        assert_eq!("AT+CLIP=1\r", client.sent_commands.get(0).unwrap());
    }

    #[tokio::test]
    async fn test_call_number() {
        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        call_number(
            &mut client,
            &mut pico,
            &String::try_from("+36301234567").unwrap(),
            100,
        )
        .await;
        assert_eq!(3, client.sent_commands.len());
        assert_eq!("AT+CHFA=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("ATD+36301234567,i;\r", client.sent_commands.get(1).unwrap());
        assert_eq!("AT+CHUP;\r", client.sent_commands.get(2).unwrap());

        assert_eq!(1, pico.sleep_calls.len());
        assert_eq!(100u64, *pico.sleep_calls.get(0).unwrap());
    }
}
