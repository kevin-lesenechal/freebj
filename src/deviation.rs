use std::str::FromStr;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum DevOverride {
    AboveEqual(f32, u8),
    UnderEqual(f32, u8),
}

#[derive(Debug, PartialEq)]
pub enum DeviationTable {
    HardTable,
    SoftTable,
    PairTable,
}

#[derive(Debug, PartialEq)]
pub struct Deviation {
    pub table: DeviationTable,
    pub row: u8,
    pub dealer: u8,
    pub action: DevOverride,
}

impl ToString for DevOverride {
    fn to_string(&self) -> String {
        match self {
            Self::AboveEqual(tc, a) => format!(">{:+}{}", tc, *a as char),
            Self::UnderEqual(tc, a) => format!("<{:+}{}", tc, *a as char),
        }
    }
}

impl FromStr for DevOverride {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^([<>])([+-]?\d+(?:\.\d+)?)(.)$").unwrap();

        if let Some(c) = regex.captures(s) {
            let tc = c[2].parse()
                .map_err(|e| format!("Invalid true count: {}", e))?;
            let action = c[3].as_bytes();

            if action.len() != 1 {
                return Err(String::from("Invalid action"));
            }
            let action = action[0];

            if !"+=DdV*?@h&SsUuEe".as_bytes().into_iter().any(|&c| c == action) {
                return Err(String::from("Invalid action"));
            }

            Ok(match c[1].as_bytes()[0] {
                b'>' => DevOverride::AboveEqual(tc, action),
                b'<' => DevOverride::UnderEqual(tc, action),
                _ => unreachable!(),
            })
        } else {
            Err(String::from("Invalid syntax"))
        }
    }
}

impl FromStr for Deviation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^(\d+|[0-9AT]/[0-9AT]|A(?:\d+|A))vs(\d+|A):(.+)$").unwrap();

        if let Some(c) = regex.captures(s) {
            let table;
            let row;

            if c[1].contains("/") {
                table = DeviationTable::PairTable;
                let pair = &c[1][0..c[1].find('/').unwrap()];
                if pair == "A" {
                    row = 9;
                } else if pair == "T" {
                    row = 0;
                } else {
                    let pair: u8 = pair.parse()
                        .map_err(|_| String::from("Invalid syntax"))?;
                    if pair < 1 || pair > 10 {
                        return Err(String::from("Invalid pair"));
                    }
                    row = 10 - pair;
                }
            } else if c[1].as_bytes()[0] == b'A' {
                table = DeviationTable::SoftTable;
                if c[1].as_bytes()[1] == b'A' {
                    row = 9;
                } else {
                    let card: u8 = c[1][1..].parse()
                        .map_err(|_| String::from("Invalid syntax"))?;
                    if card < 1 || card > 10 {
                        return Err(String::from("Invalid soft total"));
                    }
                    row = 10 - card;
                }
            } else {
                table = DeviationTable::HardTable;
                let total: u8 = c[1].parse()
                    .map_err(|_| String::from("Invalid syntax"))?;
                if total > 21 || total < 4 {
                    return Err(String::from("Invalid hard total"));
                }
                row = 20 - total;
            }

            let dealer = if &c[2] == "A" {
                1
            } else {
                c[2].parse().map_err(|e| format!("Invalid dealer card: {}", e))?
            };
            if dealer == 0 || dealer > 10 {
                return Err(String::from("Invalid dealer card"));
            }

            let action = DevOverride::from_str(&c[3])?;
            Ok(Deviation {
                table,
                row,
                dealer,
                action,
            })
        } else {
            Err(String::from("Invalid syntax"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::deviation::DevOverride::*;
    use crate::deviation::{DevOverride, Deviation};
    use crate::deviation::DeviationTable::*;
    use std::str::FromStr;

    #[test]
    fn it_parses_dev_override() {
        assert_eq!(DevOverride::from_str(">3+"),
                   Ok(DevOverride::AboveEqual(3.0, b'+')));
        assert_eq!(DevOverride::from_str(">+8D"),
                   Ok(DevOverride::AboveEqual(8.0, b'D')));
        assert_eq!(DevOverride::from_str("<0="),
                   Ok(DevOverride::UnderEqual(0.0, b'=')));
        assert_eq!(DevOverride::from_str("<0.5="),
                   Ok(DevOverride::UnderEqual(0.5, b'=')));
        assert_eq!(DevOverride::from_str("<-2="),
                   Ok(DevOverride::UnderEqual(-2.0, b'=')));

        assert_eq!(DevOverride::from_str("foo"),
                   Err(String::from("Invalid syntax")));
        assert_eq!(DevOverride::from_str(">a+"),
                   Err(String::from("Invalid syntax")));
        assert_eq!(DevOverride::from_str("=2+"),
                   Err(String::from("Invalid syntax")));
        assert_eq!(DevOverride::from_str(">3#"),
                   Err(String::from("Invalid action")));
    }

    #[test]
    fn it_parses_deviation() {
        assert_eq!(Deviation::from_str("16vs10:>+1="),
                   Ok(Deviation{
                       table: HardTable,
                       row: 4,
                       dealer: 10,
                       action: AboveEqual(1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("20vs2:>-1="),
                   Ok(Deviation{
                       table: HardTable,
                       row: 0,
                       dealer: 2,
                       action: AboveEqual(-1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("4vsA:>-1="),
                   Ok(Deviation{
                       table: HardTable,
                       row: 16,
                       dealer: 1,
                       action: AboveEqual(-1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("A6vs8:>-1="),
                   Ok(Deviation{
                       table: SoftTable,
                       row: 4,
                       dealer: 8,
                       action: AboveEqual(-1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("A10vs8:>-1="),
                   Ok(Deviation{
                       table: SoftTable,
                       row: 0,
                       dealer: 8,
                       action: AboveEqual(-1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("A2vsA:>-1="),
                   Ok(Deviation{
                       table: SoftTable,
                       row: 8,
                       dealer: 1,
                       action: AboveEqual(-1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("AAvs8:>-1="),
                   Ok(Deviation{
                       table: SoftTable,
                       row: 9,
                       dealer: 8,
                       action: AboveEqual(-1.0, b'=')
                   }));
        assert_eq!(Deviation::from_str("7/7vs7:<+1D"),
                   Ok(Deviation{
                       table: PairTable,
                       row: 3,
                       dealer: 7,
                       action: UnderEqual(1.0, b'D')
                   }));
        assert_eq!(Deviation::from_str("A/Avs7:<+1D"),
                   Ok(Deviation{
                       table: PairTable,
                       row: 9,
                       dealer: 7,
                       action: UnderEqual(1.0, b'D')
                   }));
        assert_eq!(Deviation::from_str("T/Tvs7:<+1D"),
                   Ok(Deviation{
                       table: PairTable,
                       row: 0,
                       dealer: 7,
                       action: UnderEqual(1.0, b'D')
                   }));
    }
}
