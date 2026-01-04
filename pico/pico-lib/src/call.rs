use alloc::format;
use alloc::string::ToString;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::heapless::String;
use embassy_time::Duration;

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

pub async fn call_number<T: atat::asynch::AtatClient>(
    client: &mut T,
    number: &'static str,
    duration: Duration,
) {
    {
        let _ = LogBE::new("AtSwapAudioChannelsWrite".to_string());
        let r = client
            .send(&AtSwapAudioChannelsWrite {
                n: AudioChannels::Main,
            })
            .await;
        match r {
            Ok(_) => log::info!("OK"),
            Err(e) => log::info!("ERROR: {:?}", e),
        }
    }

    {
        let _ = LogBE::new("AtDialNumber".to_string());
        let r = client
            .send(&AtDialNumber {
                number: String::<16>::try_from(number).unwrap(),
            })
            .await;
        match r {
            Ok(_) => log::info!("OK"),
            Err(e) => log::info!("ERROR: {:?}", e),
        }
    }

    {
        LogBE::new(format!("Sleeping for duration {}", duration));
        // todo unit test: undefined symbol: _embassy_time_schedule_wake (maybe? https://github.com/esp-rs/esp-hal/issues/1435)
        // or just create a sleeper trait
        //Timer::after(duration).await;
    }

    {
        LogBE::new("AtHangup".to_string());
        let r = client.send(&AtHangup).await;
        match r {
            Ok(_) => log::info!("OK"),
            Err(e) => log::info!("ERROR: {:?}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{at, cmd_serialization_tests};

    use super::*;
    use alloc::collections::vec_deque::VecDeque;
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

    use alloc::string::String as AString;

    // TODO: try mock_all
    struct ClientMock {
        sent_commands: VecDeque<AString>,
        //resuts: VecDeque<Result<Cmd::Response, atat::Error>>,
    }

    impl atat::asynch::AtatClient for ClientMock {
        async fn send<Cmd: AtatCmd>(&mut self, cmd: &Cmd) -> Result<Cmd::Response, atat::Error> {
            let mut buffer = crate::at::tests::zeros();
            cmd.write(&mut buffer);
            let tmp = String::from_utf8(buffer).unwrap();
            let trimmed = tmp.trim_matches(char::from(0));
            self.sent_commands.push_back(AString::from(trimmed));
            //self.resuts.pop_front()
            Err(atat::Error::Error)
        }
    }

    #[tokio::test]
    async fn test_call_number() {
        at::tests::init_env_logger();

        let mut client = ClientMock {
            sent_commands: VecDeque::new(),
            //results: VecDeque::new(),
        };
        call_number(&mut client, "+36301234567", Duration::from_millis(1)).await;
        assert_eq!(3, client.sent_commands.len());
    }
}
