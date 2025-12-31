use atat::atat_derive::AtatCmd;
use atat::atat_derive::AtatResp;
use atat::heapless::String;

#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ATE1", NoResponse, cmd_prefix = "", termination = "\r\n")]
pub struct ATE;

#[derive(Clone, Debug, AtatCmd)]
#[at_cmd(
    "AT",
    NoResponse,
    cmd_prefix = "",
    timeout_ms = 5000,
    termination = "\r\n"
)]
pub struct AT;

#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CGREG?", CGREGText, termination = "\r\n")]
pub struct CGREG;

#[derive(Clone, AtatResp)]
pub struct NoResponse;

#[derive(Clone, AtatResp)]
pub struct CGREGText {
    #[at_arg(position = 0)]
    pub text: String<64>,
}

#[cfg(test)]
extern crate std;

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
    fn test_at() {
        let k = AT {};
        let mut buffer = zeros();
        assert_eq!(4, k.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "AT\r\n"
        );
    }

    #[test]
    fn test_ate() {
        let k = ATE {};
        let mut buffer = zeros();
        assert_eq!(6, k.write(&mut buffer));
        assert_eq!(
            String::from_utf8(buffer)
                .unwrap()
                .trim_matches(char::from(0)),
            "ATE1\r\n"
        );
    }
}
