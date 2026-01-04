use atat::atat_derive::AtatResp;

#[derive(Debug, Clone, AtatResp)]
pub struct NoResponse;

#[cfg(test)]
extern crate std;

#[cfg(test)]
pub mod tests {
    use atat::heapless::Vec;

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
                let (cmd, len, text) = $value;
                let mut buffer = crate::at::tests::zeros();
                assert_eq!(len, cmd.write(&mut buffer));
                assert_eq!(
                    String::from_utf8(buffer)
                        .unwrap()
                        .trim_matches(char::from(0)),
                    text
                );
            }
        )*
        }
    }

    pub fn init_env_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}
