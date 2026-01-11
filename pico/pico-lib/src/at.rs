use atat::atat_derive::AtatResp;
use defmt::Format;

#[derive(Debug, Format, Clone, AtatResp)]
pub struct NoResponse;

pub trait PicoHW {
    fn sleep(&mut self, millis: u64) -> impl core::future::Future<Output = ()> + Send;
    fn set_led_high(&mut self);
    fn set_led_low(&mut self);
    fn restart_module(&mut self) -> impl core::future::Future<Output = ()> + Send;
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
pub mod tests {
    use alloc::string::String as AString;
    use alloc::{collections::vec_deque::VecDeque, vec::Vec as AVec};
    use atat::{AtatCmd, heapless::Vec};

    use crate::at::PicoHW;

    pub fn zeros() -> Vec<u8, 127> {
        let mut buffer = Vec::<u8, 127>::new();
        for _ in 0..127 {
            let _ = buffer.push(0u8);
        }
        return buffer;
    }

    #[macro_export]
    macro_rules! cmd_serialization_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (cmd, text) = $value;
                let mut buffer = crate::at::tests::zeros();
                assert_eq!(text.len(), cmd.write(&mut buffer));
                assert_eq!(
                    atat::heapless::String::from_utf8(buffer)
                        .unwrap()
                        .trim_matches(char::from(0)),
                    text
                );
            }
        )*
        }
    }

    #[derive(Default)]
    pub struct ClientMock<'a> {
        pub sent_commands: VecDeque<AString>,
        pub results: VecDeque<Result<&'a [u8], atat::InternalError<'a>>>,
    }

    impl atat::asynch::AtatClient for ClientMock<'_> {
        async fn send<Cmd: AtatCmd>(&mut self, cmd: &Cmd) -> Result<Cmd::Response, atat::Error> {
            let mut buffer = crate::at::tests::zeros();
            cmd.write(&mut buffer);
            let tmp = atat::heapless::String::from_utf8(buffer).unwrap();
            let trimmed = tmp.trim_matches(char::from(0));
            self.sent_commands.push_back(AString::from(trimmed));
            cmd.parse(self.results.pop_front().expect("missing result"))
        }
    }

    #[derive(Default)]
    pub struct PicoMock {
        pub sleep_calls: AVec<u64>,
        pub set_led_high_calls: u32,
        pub set_led_low_calls: u32,
        pub restart_module_calls: u32,
    }

    impl PicoHW for PicoMock {
        async fn sleep(&mut self, millis: u64) {
            self.sleep_calls.push(millis);
        }

        fn set_led_high(&mut self) {
            self.set_led_high_calls += 1;
        }

        fn set_led_low(&mut self) {
            self.set_led_low_calls += 1;
        }

        async fn restart_module(&mut self) {
            self.restart_module_calls += 1;
        }
    }
}
