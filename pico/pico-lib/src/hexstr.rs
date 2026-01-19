use alloc::{fmt, format, vec::Vec};
use atat::{
    AtatLen,
    heapless::String,
    serde_at::serde::{self, Deserialize, Serialize, de::Visitor},
};
use core::{num::ParseIntError, str::FromStr};
use defmt::Format;

pub fn decode_hex_u8(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn decode_utf8_hex_string<const N: usize>(v: &[u8]) -> Result<String<N>, &'static str> {
    let hex_str = core::str::from_utf8(&v).map_err(|_o| -> &'static str { "utf-8 error" })?;
    let bytes = decode_hex_u8(&hex_str).map_err(|_o| -> &'static str { "decode_hex_u8 error" })?;
    let utf8_str =
        core::str::from_utf8(&bytes).map_err(|_o| -> &'static str { "from_utf8 error" })?;
    String::from_str(utf8_str).map_err(|_o| -> &'static str { "from_str error" })
}

pub fn encode_utf8_hex_string<const N: usize>(v: &[u8]) -> Result<String<N>, &'static str> {
    let mut hex_str = String::<N>::new();
    for c in v {
        let s = format!("{:02X}", c);
        hex_str
            .push_str(s.as_str())
            .map_err(|_o| -> &'static str { "push_str error" })?;
    }
    Ok(hex_str)
}

pub fn decode_hex_u16(s: &str) -> Result<Vec<u16>, ParseIntError> {
    (0..s.len())
        .step_by(4)
        .map(|i| u16::from_str_radix(&s[i..i + 4], 16))
        .collect()
}

pub fn decode_utf16_hex_string<const N: usize>(v: &[u8]) -> Result<String<N>, &'static str> {
    let hex_str = core::str::from_utf8(&v).map_err(|_o| -> &'static str { "utf-8 error" })?;
    let bytes =
        decode_hex_u16(&hex_str).map_err(|_o| -> &'static str { "decode_hex_u16 error" })?;
    let utf16_str = alloc::string::String::from_utf16(&bytes)
        .map_err(|_o| -> &'static str { "from_utf16 error" })?;
    String::from_str(utf16_str.as_str()).map_err(|_o| -> &'static str { "from_str error" })
}

pub fn encode_utf16_hex_string<const N: usize>(v: &[u8]) -> Result<String<N>, &'static str> {
    let utf8_str = core::str::from_utf8(v).map_err(|_o| -> &'static str { "from_utf8 error" })?;
    let s = String::<N>::from_str(utf8_str).map_err(|_o| -> &'static str { "from_str error" })?;
    let v: Vec<u16> = s.encode_utf16().collect();
    let mut hex_str = String::<N>::new();
    for c in v {
        let s = format!("{:04X}", c);
        hex_str
            .push_str(s.as_str())
            .map_err(|_o| -> &'static str { "push_str error" })?;
    }
    Ok(hex_str)
}

struct HexStringVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for HexStringVisitor<N> {
    type Value = (String<N>, bool);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a hex string in utf-8 / utf-16 format")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut s = core::str::from_utf8(v).map_err(serde::de::Error::custom)?;
        let mut quoted = false;
        if s.starts_with('"') && s.ends_with('"') {
            quoted = true;
            let mut chars = s.chars();
            chars.next();
            chars.next_back();
            s = chars.as_str();
        }

        let decoded = decode_utf16_hex_string(s.as_bytes());
        match decoded {
            Ok(d) => {
                if d.len() > N {
                    return Err(serde::de::Error::custom("source string too long"));
                }
                return Ok((d, quoted));
            }
            Err(e) => {
                return Err(serde::de::Error::custom(e));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Format)]
pub struct UCS2HexString<const N: usize> {
    pub text: String<N>,
    pub quoted: bool,
}

impl<'de, const N: usize> Deserialize<'de> for UCS2HexString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let out = deserializer.deserialize_bytes(HexStringVisitor)?;

        Ok(Self {
            text: out.0,
            quoted: out.1,
        })
    }
}

impl<const N: usize> From<&str> for UCS2HexString<N> {
    fn from(s: &str) -> Self {
        Self {
            text: String::<N>::from_str(s).unwrap(),
            quoted: false,
        }
    }
}

impl<const N: usize> fmt::Display for UCS2HexString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl<const N: usize> Serialize for UCS2HexString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !self.quoted {
            todo!()
        }

        let v: String<512> = encode_utf16_hex_string(self.text.as_bytes())
            .map_err(|_o| -> S::Error { serde::ser::Error::custom("encode utf-16 error") })?;
        return serializer.serialize_str(v.as_str());
    }
}

impl<const N: usize> AtatLen for UCS2HexString<N> {
    const LEN: usize = N * 2;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_encode_utf8_hex_string() {
        assert_eq!(
            String::<30>::try_from("").unwrap(),
            encode_utf8_hex_string::<30>(b"").unwrap()
        );
        assert_eq!(
            String::<30>::try_from("3030303036").unwrap(),
            encode_utf8_hex_string::<30>(b"00006").unwrap()
        );
        assert_eq!(
            String::<30>::try_from("2B3336333031323334353637").unwrap(),
            encode_utf8_hex_string::<30>(b"+36301234567").unwrap()
        );
        assert_eq!(
            String::<50>::try_from("24744154412F6C6F636174696F6E2F3132333435").unwrap(),
            encode_utf8_hex_string::<50>(b"$tATA/location/12345").unwrap()
        );
        assert_eq!(
            String::<100>::try_from(
                "4BC3B6737AC3B66E6AC3BC6B2E0AC39C6476C3B67A6C657474656C2059657474656C2E"
            )
            .unwrap(),
            encode_utf8_hex_string::<100>(
                String::<100>::try_from("Köszönjük.\nÜdvözlettel Yettel.")
                    .unwrap()
                    .as_bytes()
            )
            .unwrap()
        );
        assert_eq!(
            String::<100>::try_from("54616DC3A1732044C3B66DC5916B20F09F988E").unwrap(),
            encode_utf8_hex_string::<100>(
                String::<100>::try_from("Tamás Dömők 😎")
                    .unwrap()
                    .as_bytes()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_decode_utf8_hex_string() {
        assert_eq!(
            String::<30>::try_from("").unwrap(),
            decode_utf8_hex_string::<30>(b"").unwrap()
        );
        assert_eq!(
            String::<30>::try_from("00006").unwrap(),
            decode_utf8_hex_string::<30>(b"3030303036").unwrap()
        );
        assert_eq!(
            String::<30>::try_from("+36301234567").unwrap(),
            decode_utf8_hex_string::<30>(b"2B3336333031323334353637").unwrap()
        );
        assert_eq!(
            String::<30>::try_from("$tATA/location/12345").unwrap(),
            decode_utf8_hex_string::<30>(b"24744154412F6C6F636174696F6E2F3132333435").unwrap()
        );
        assert_eq!(
            String::<100>::try_from("Köszönjük.\nÜdvözlettel Yettel.").unwrap(),
            decode_utf8_hex_string::<100>(
                b"4BC3B6737AC3B66E6AC3BC6B2E0AC39C6476C3B67A6C657474656C2059657474656C2E"
            )
            .unwrap()
        );
        assert_eq!(
            String::<100>::try_from("Tamás Dömők 😎").unwrap(),
            decode_utf8_hex_string::<100>(b"54616DC3A1732044C3B66DC5916B20F09F988E").unwrap()
        );
    }

    #[test]
    fn test_encode_utf16_hex_string() {
        assert_eq!(
            String::<100>::try_from("").unwrap(),
            encode_utf16_hex_string::<100>(b"").unwrap()
        );
        assert_eq!(
            String::<100>::try_from("00300030003000300036").unwrap(),
            encode_utf16_hex_string::<100>(b"00006").unwrap()
        );
        assert_eq!(
            String::<100>::try_from("002B00330036003300300031003200330034003500360037").unwrap(),
            encode_utf16_hex_string::<100>(b"+36301234567").unwrap()
        );
        assert_eq!(
            String::<100>::try_from(
                "00240074004100540041002F006C006F0063006100740069006F006E002F00310032003300340035"
            )
            .unwrap(),
            encode_utf16_hex_string::<100>(b"$tATA/location/12345").unwrap()
        );
        assert_eq!(
            String::<500>::try_from(
                "004B00F60073007A00F6006E006A00FC006B002E000A00DC0064007600F6007A006C0065007400740065006C002000590065007400740065006C002E"
            )
            .unwrap(),
            encode_utf16_hex_string::<500>(
                String::<500>::try_from("Köszönjük.\nÜdvözlettel Yettel.")
                    .unwrap()
                    .as_bytes()
            )
            .unwrap()
        );
        assert_eq!(
            String::<500>::try_from("00540061006D00E100730020004400F6006D0151006B0020D83DDE0E")
                .unwrap(),
            encode_utf16_hex_string::<500>(
                String::<500>::try_from("Tamás Dömők 😎")
                    .unwrap()
                    .as_bytes()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_decode_utf16_hex_string() {
        assert_eq!(
            String::<500>::try_from("").unwrap(),
            decode_utf16_hex_string::<30>(b"").unwrap()
        );
        assert_eq!(
            String::<500>::try_from("00006").unwrap(),
            decode_utf16_hex_string::<500>(b"00300030003000300036").unwrap()
        );
        assert_eq!(
            String::<500>::try_from("+36301234567").unwrap(),
            decode_utf16_hex_string::<500>(b"002B00330036003300300031003200330034003500360037")
                .unwrap()
        );
        assert_eq!(
            String::<500>::try_from("$tATA/location/12345").unwrap(),
            decode_utf16_hex_string::<500>(
                b"00240074004100540041002F006C006F0063006100740069006F006E002F00310032003300340035"
            )
            .unwrap()
        );
        assert_eq!(
            String::<500>::try_from("Köszönjük.\nÜdvözlettel Yettel.").unwrap(),
            decode_utf16_hex_string::<500>(
                b"004B00F60073007A00F6006E006A00FC006B002E000A00DC0064007600F6007A006C0065007400740065006C002000590065007400740065006C002E"
            )
            .unwrap()
        );
        assert_eq!(
            String::<500>::try_from("Tamás Dömők 😎").unwrap(),
            decode_utf16_hex_string::<500>(
                b"00540061006D00E100730020004400F6006D0151006B0020D83DDE0E"
            )
            .unwrap()
        );
    }
}
