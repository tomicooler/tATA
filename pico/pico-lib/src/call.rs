use alloc::format;
use atat::AtatCmd;
use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatEnum;
use atat::heapless::String;

use crate::at::NoResponse;

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

#[cfg(test)]
mod tests {
    use super::*;
    use atat::AtatCmd;
    use atat::heapless::Vec;

    fn zeros() -> Vec<u8, 127> {
        let mut buffer = Vec::<u8, 127>::new();
        for _ in 0..127 {
            let _ = buffer.push(0u8);
        }
        return buffer;
    }

    #[test]
    fn test_at_swap_audio_channels_write() {
        let cmd = AtSwapAudioChannelsWrite {
            n: AudioChannels::Main,
        };
        let mut buffer = zeros();
        assert_eq!(10, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+CHFA=1\r"
        );
    }

    #[test]
    fn test_at_dial_number() {
        let cmd = AtDialNumber {
            number: String::try_from("+361234567").unwrap(),
        };
        let mut buffer = zeros();
        assert_eq!(17, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "ATD+361234567,i;\r"
        );
    }

    #[test]
    fn test_at_hangup() {
        let cmd = AtHangup;
        let mut buffer = zeros();
        assert_eq!(9, cmd.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT+CHUP;\r"
        );
    }
}
