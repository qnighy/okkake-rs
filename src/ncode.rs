use std::fmt;
use std::str::{self, FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeDisplay, DeserializeFromStr,
)]
pub(crate) struct Ncode(u32);

impl fmt::Display for Ncode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            f.write_str("n0000a")?;
            return Ok(());
        }
        let hi = (self.0 - 1) / 9999;
        let lo = (self.0 - 1) % 9999 + 1;
        write!(f, "n{}", lo)?;
        {
            let mut x = hi;
            let mut buf = [0; 20];
            let mut i = 0;
            while i == 0 || x > 0 {
                buf[i] = b'a' + (x % 26) as u8;
                i += 1;
                x /= 26;
            }
            buf[..i].reverse();
            f.write_str(str::from_utf8(&buf[..i]).unwrap())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NcodeParseError;

impl fmt::Display for NcodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid ncode")
    }
}

impl FromStr for Ncode {
    type Err = NcodeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(s.starts_with("n") || s.starts_with("N")) {
            return Err(NcodeParseError);
        }
        let lo_part = s.get(1..5).ok_or(NcodeParseError)?;
        if !lo_part.as_bytes().iter().all(|&ch| ch.is_ascii_digit()) {
            return Err(NcodeParseError);
        }
        let lo = lo_part.parse::<u32>().map_err(|_| NcodeParseError)?;
        let alpha_part = s.get(5..).ok_or(NcodeParseError)?;
        let hi = {
            let mut hi: u32 = 0;
            for &ch in alpha_part.as_bytes() {
                if !ch.is_ascii_alphabetic() {
                    return Err(NcodeParseError);
                }
                // hi = hi * 26 - (ch - b'a')
                hi = hi
                    .checked_mul(26)
                    .ok_or(NcodeParseError)?
                    .checked_add((ch.to_ascii_lowercase() - b'a') as u32)
                    .ok_or(NcodeParseError)?;
            }
            hi
        };
        // let num = hi * 9999 + lo
        let num = hi
            .checked_mul(9999)
            .ok_or(NcodeParseError)?
            .checked_add(lo)
            .ok_or(NcodeParseError)?;
        Ok(Ncode(num))
    }
}

#[test]
fn test_ncode_roundtrip() {
    let cases = ["n4830bu", "n9999x", "n1234a", "n0000a", "n0001a"];
    for &case in cases.iter() {
        assert_eq!(case.parse::<Ncode>().unwrap().to_string(), case);
    }
}

#[test]
fn test_ncode_normalize() {
    let cases = [
        ("N4830Bu", "n4830bu"),
        ("n0000y", "n9999x"),
        ("n0000", "n0000a"),
        ("n1010", "n1010a"),
        ("n4830aaaaaabu", "n4830bu"),
    ];
    for &(input, output) in cases.iter() {
        assert_eq!(input.parse::<Ncode>().unwrap().to_string(), output);
    }
}

#[test]
fn test_ncode_parse_error() {
    let cases = ["", "n", "n0", "n00", "n000", "n00000", "na", "naa"];
    for &case in cases.iter() {
        assert!(
            case.parse::<Ncode>().is_err(),
            "Expected parse error: {:?}",
            case
        );
    }
}
