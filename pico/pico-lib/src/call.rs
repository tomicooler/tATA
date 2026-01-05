use alloc::format;
use alloc::string::ToString;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::heapless::String;

use crate::at::NoResponse;
use crate::utils::LogBE;

// 6.2.19 AT+CHFA Swap the Audio Channels
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CHFA", NoResponse)]
pub struct AtSwapAudioChannelsWrite {
    pub n: AudioChannels,
}

#[derive(Debug, Clone, PartialEq, AtatEnum)]
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
    pub number: String<16>,
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
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CHUP;", NoResponse)]
pub struct AtHangup;

pub async fn call_number<T: atat::asynch::AtatClient, U: crate::at::PicoHW>(
    client: &mut T,
    pico: &mut U,
    number: &'static str,
    duration_millis: u64,
) {
    {
        let _l = LogBE::new("AtSwapAudioChannelsWrite".to_string());
        let r = client
            .send(&AtSwapAudioChannelsWrite {
                n: AudioChannels::Main,
            })
            .await;
        match r {
            Ok(_) => log::info!("  OK"),
            Err(e) => log::info!("  ERROR: {:?}", e),
        }
    }

    {
        let _l = LogBE::new("AtDialNumber".to_string());
        let r = client
            .send(&AtDialNumber {
                number: String::<16>::try_from(number).unwrap(),
            })
            .await;
        match r {
            Ok(_) => log::info!("  OK"),
            Err(e) => log::info!("  ERROR: {:?}", e),
        }
    }

    {
        let _l = LogBE::new("Sleeping".to_string());
        pico.sleep(duration_millis).await;
    }

    {
        let _l = LogBE::new("AtHangup".to_string());
        let r = client.send(&AtHangup).await;
        match r {
            Ok(_) => log::info!("  OK"),
            Err(e) => log::info!("  ERROR: {:?}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{at, cmd_serialization_tests};

    use super::*;
    use atat::AtatCmd;

    cmd_serialization_tests! {
        test_at_swap_audio_channels_write: (
            AtSwapAudioChannelsWrite {
                n: AudioChannels::Main,
            },
            10,
            "AT+CHFA=1\r",
        ),
        test_at_dial_number: (
            AtDialNumber {
                number: String::try_from("+361234567").unwrap(),
            },
            17,
            "ATD+361234567,i;\r",
        ),
        test_at_hangup: (
            AtHangup,
            9,
            "AT+CHUP;\r",
        ),
    }

    #[tokio::test]
    async fn test_call_number() {
        at::tests::init_env_logger();

        let mut client = crate::at::tests::ClientMock::default();
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));
        client.results.push_back(Ok("".as_bytes()));

        let mut pico = crate::at::tests::PicoMock::default();
        call_number(&mut client, &mut pico, "+36301234567", 100).await;
        assert_eq!(3, client.sent_commands.len());
        assert_eq!("AT+CHFA=1\r", client.sent_commands.get(0).unwrap());
        assert_eq!("ATD+36301234567,i;\r", client.sent_commands.get(1).unwrap());
        assert_eq!("AT+CHUP;\r", client.sent_commands.get(2).unwrap());

        assert_eq!(1, pico.sleep_calls.len());
        assert_eq!(100u64, *pico.sleep_calls.get(0).unwrap());
    }
}
