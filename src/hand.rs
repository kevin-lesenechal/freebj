use crate::card::Card;
use arrayvec::ArrayVec;
use std::ops::Index;
use std::fmt::Debug;
use bitflags::_core::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Hand {
    cards:          ArrayVec<[Card; 16]>,
    value:          u8,
    is_soft:        bool,
    is_busted:      bool,

    /// Owner's ID, useful when a player has many split hands
    pub id:         u8,

    /// The bet associated with the hand
    pub bet:        f64,

    /// Whether this hand was doubled down
    doubled:        bool,

    /// Whether this hand is the result of a pair split
    split:          bool,

    /// Whether this hand was surrendered or not
    surrendered:    bool,

    /// Whether this hand was insured or not
    insured:        bool,
}

impl Hand {
    /// Creates an empty hand of cards
    pub fn new() -> Self {
        Self {
            cards:      ArrayVec::new(),
            value:      0,
            is_soft:    false,
            is_busted:  false,
            id:         0,
            bet:        1.0,
            doubled:    false,
            split:      false,
            surrendered: false,
            insured:    false,
        }
    }

    /// Returns the current number of cards
    pub fn count(&self) -> usize {
        self.cards.len()
    }

    /// Adds a new card to the hand
    pub fn add(&mut self, card: Card) {
        assert!(card.0 > 0 && card.0 < 11);

        if card.0 == 1 {
            if self.value <= 10 {
                self.is_soft = true;
                self.value += 11;
            } else {
                self.value += 1;
            }
        } else {
            self.value += card.0;
        }

        if self.value > 21 {
            if self.is_soft {
                self.value -= 10;
                self.is_soft = false;
            } else {
                self.is_busted = true;
            }
        }

        self.cards.push(card);
    }

    pub fn iter(&self) -> impl Iterator<Item=&Card> {
        self.cards.iter()
    }

    /// Returns the point value of the hand, aces are always counted as 11 if
    /// they do no bust the hand.
    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn double_down(&mut self) {
        assert!(!self.doubled);
        assert!(!self.surrendered);
        self.doubled = true;
    }

    pub fn split(&mut self) {
        assert!(!self.split);
        assert!(!self.doubled);
        assert!(!self.surrendered);
        self.split = true;
    }

    pub fn surrender(&mut self) {
        assert!(!self.surrendered);
        assert!(!self.split);
        assert!(!self.doubled);
        self.surrendered = true;
    }

    pub fn insure(&mut self) {
        assert!(!self.insured);
        assert!(!self.surrendered);
        self.insured = true;
    }

    /// Returns whether the hand is a natural blackjack or not
    pub fn is_bj(&self) -> bool {
        !self.split && self.count() == 2 && self.value == 21
    }

    /// Returns whether the hand is busted or not
    pub fn is_busted(&self) -> bool { self.is_busted }

    /// Returns whether the hand is soft
    pub fn is_soft(&self) -> bool { self.is_soft }

    pub fn is_doubled(&self) -> bool { self.doubled }

    pub fn is_split(&self) -> bool { self.split }

    pub fn is_surrendered(&self) -> bool { self.surrendered }

    pub fn is_insured(&self) -> bool { self.insured }
}

impl Index<usize> for Hand {
    type Output = Card;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cards[index]
    }
}

impl From<&[u8]> for Hand {
    fn from(cards: &[u8]) -> Self {
        let mut hand = Hand::new();
        for &card in cards {
            hand.add(Card(card));
        }
        hand
    }
}

impl From<&[Card]> for Hand {
    fn from(cards: &[Card]) -> Self {
        let mut hand = Hand::new();
        for &card in cards {
            hand.add(card);
        }
        hand
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[")?;

        for (i, card) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:2}", card)?;
        }

        write!(f, "] = {}", self.value)?;

        if self.doubled { write!(f, ", doubled")?; }
        if self.split { write!(f, ", split")?; }
        if self.insured { write!(f, ", insured")?; }
        if self.surrendered { write!(f, ", surrendered")?; }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::Hand;
    use crate::card::Card;

    #[test]
    fn it_returns_the_number_of_cards() {
        let empty = Hand::new();
        assert_eq!(empty.count(), 0);

        let mut hand = Hand::new();
        hand.add(Card(5));
        hand.add(Card(8));
        assert_eq!(hand.count(), 2);
    }

    #[test]
    #[should_panic]
    fn it_panicks_when_adding_17_cards() {
        let mut hand = Hand::new();

        for _ in 0..17 {
            hand.add(Card(5));
        }
    }

    #[test]
    fn it_can_be_indexed() {
        let mut hand = Hand::new();
        hand.add(Card(10));
        hand.add(Card(7));
        hand.add(Card(4));

        assert_eq!(hand[0], Card(10));
        assert_eq!(hand[1], Card(7));
        assert_eq!(hand[2], Card(4));
    }

    #[test]
    fn it_can_be_iterated_over() {
        let mut hand = Hand::new();
        hand.add(Card(10));
        hand.add(Card(7));
        hand.add(Card(4));

        let mut iter = hand.iter();
        assert_eq!(iter.next(), Some(&Card(10)));
        assert_eq!(iter.next(), Some(&Card(7)));
        assert_eq!(iter.next(), Some(&Card(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn it_returns_the_value_of_hard_hands() {
        test_hand(&[10, 5],         15, Some(false), None, None);
        test_hand(&[7, 2, 2, 8],    19, Some(false), None, None);
        test_hand(&[10, 10, 10],    30, Some(false), None, None);
    }

    #[test]
    fn it_handles_soft_hands() {
        test_hand(&[1, 8],      19, Some(true), None, None);
        test_hand(&[1, 2, 4],   17, Some(true), None, None);
        test_hand(&[1, 1],      12, Some(true), None, None);
        test_hand(&[5, 1],      16, Some(true), None, None);
        test_hand(&[1, 10],     21, Some(true), None, None);
        test_hand(&[10, 1],     21, Some(true), None, None);
        test_hand(&[2, 8, 1],   21, Some(true), None, None);
        test_hand(&[1, 3, 7],   21, Some(true), None, None);
    }

    #[test]
    fn it_handles_hard_hands_with_aces() {
        test_hand(&[10, 8, 1],          19, Some(false), None, None);
        test_hand(&[10, 8, 1, 1, 1],    21, Some(false), None, None);
        test_hand(&[1, 7, 4],           12, Some(false), None, None);
        test_hand(&[5, 1, 4, 7],        17, Some(false), None, None);
    }

    #[test]
    fn it_detects_busted_hands() {
        test_hand(&[10, 6, 6],      22, None, Some(true), None);
        test_hand(&[1, 6, 6, 9],    22, None, Some(true), None);
        test_hand(&[7, 7, 7, 1],    22, None, Some(true), None);
        test_hand(&[10, 7],         17, None, Some(false), None);
        test_hand(&[10, 1],         21, None, Some(false), None);
        test_hand(&[7, 7, 7],       21, None, Some(false), None);
    }

    #[test]
    fn it_detects_blackjacks() {
        test_hand(&[1, 10],     21, Some(true), Some(false), Some(true));
        test_hand(&[10, 1],     21, Some(true), Some(false), Some(true));
        test_hand(&[7, 7, 7],   21, Some(false), Some(false), Some(false));
    }

    fn test_hand(cards: &[u8],
                 value: u8,
                 is_soft: Option<bool>,
                 is_busted: Option<bool>,
                 is_bj: Option<bool>) {
        let mut hand = Hand::new();
        for card in cards {
            hand.add(Card(*card));
        }

        assert_eq!(hand.value(), value, "{:?}", hand);

        match is_soft {
            Some(true) => assert!(hand.is_soft(),
                                  "Expected SOFT hand: {:?}", hand),
            Some(false) => assert!(!hand.is_soft(),
                                   "Expected HARD hand: {:?}", hand),
            None => (),
        }

        match is_busted {
            Some(true) => assert!(hand.is_busted(),
                                  "Expected BUSTED: {:?}", hand),
            Some(false) => assert!(!hand.is_busted(),
                                   "Expected not BUSTED: {:?}", hand),
            None => (),
        }

        match is_bj {
            Some(true) => assert!(hand.is_bj(),
                                  "Expected BLACKJACK: {:?}", hand),
            Some(false) => assert!(!hand.is_bj(),
                                   "Expected not BLACKJACK: {:?}", hand),
            None => (),
        }
    }
}
