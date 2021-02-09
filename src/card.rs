use std::fmt;
use bitflags::_core::convert::TryFrom;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Card(pub u8);

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            write!(f, "A")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl TryFrom<&str> for Card {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "A" {
            Ok(Card(1))
        } else {
            let n: u8 = value.parse().map_err(|_| "Invalid card")?;
            if n < 2 || n > 10 {
                Err("Invalid card")
            } else {
                Ok(Card(n))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::card::Card;
    use std::convert::TryFrom;

    #[test]
    fn it_converts_from_str() {
        assert_eq!(Card::try_from("2"), Ok(Card(2)));
        assert_eq!(Card::try_from("5"), Ok(Card(5)));
        assert_eq!(Card::try_from("10"), Ok(Card(10)));
        assert_eq!(Card::try_from("A"), Ok(Card(1)));

        assert_eq!(Card::try_from("11"), Err("Invalid card"));
        assert_eq!(Card::try_from("96558"), Err("Invalid card"));
        assert_eq!(Card::try_from("0"), Err("Invalid card"));
        assert_eq!(Card::try_from("1"), Err("Invalid card"));
        assert_eq!(Card::try_from(" 5"), Err("Invalid card"));
        assert_eq!(Card::try_from("8 "), Err("Invalid card"));
        assert_eq!(Card::try_from("AA"), Err("Invalid card"));
    }
}
