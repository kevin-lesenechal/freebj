use std::fmt;
use rand::rngs::SmallRng;
use rand::{SeedableRng, Rng};
use rand::seq::SliceRandom;

use crate::card::Card;
use crate::shoe::CardShoe;

#[derive(Debug)]
pub struct StandardShoe {
    cards: Vec<Card>,
    decks: u32,
    min_cards: usize,
    needs_reshuffle: bool,
    running_count: i32,
    rng: SmallRng,
}

impl StandardShoe {
    pub fn non_shuffled(decks: u32, pen_cards: u32) -> StandardShoe {
        assert!(decks > 0);

        let cards = Vec::with_capacity(decks as usize * 52);

        let mut shoe = StandardShoe {
            cards,
            decks,
            min_cards: (decks as usize * 52) - pen_cards as usize,
            needs_reshuffle: false,
            running_count: 0,
            rng: SmallRng::from_entropy(),
        };
        shoe.fill_cards();

        shoe
    }

    pub fn shuffled(decks: u32, pen_cards: u32) -> StandardShoe {
        let mut shoe = Self::non_shuffled(decks, pen_cards);
        shoe.reshuffle();

        shoe
    }

    fn fill_cards(&mut self) {
        for _ in 0..self.decks {
            for _ in 0..4 {
                for c in 1..=9 {
                    self.cards.push(Card(c));
                }
                for _ in 0..4 {
                    self.cards.push(Card(10));
                }
            }
        }
    }

    fn card_removed(&mut self, card: Card) {
        match card.0 {
            2..=6 => self.running_count += 1,
            1 | 10 => self.running_count -= 1,
            _ => (),
        }

        if self.cards.len() <= self.min_cards {
            self.needs_reshuffle = true;
        }
    }

    fn remove_high_card(&mut self) -> Card {
        let (card, card_alt) = if self.rng.gen_range(0..4) == 0 {
            (Card(1), Card(10))
        } else {
            (Card(10), Card(1))
        };
        self.try_pick_first(card).or_else(|| {
            self.try_pick_first(card_alt)
        }).expect("Not enough high cards to reach desired true count")
    }

    fn remove_low_card(&mut self) -> Card {
        let card_orig = self.rng.gen_range(2..7);
        let mut card = card_orig;
        loop {
            if let Some(c) = self.try_pick_first(Card(card)) {
                break c;
            }

            card += 1;
            if card >= 7 {
                card = 2;
            }

            if card == card_orig {
                panic!("Not enough low cards to reach desired true count");
            }
        }
    }
}

impl CardShoe for StandardShoe {
    #[inline]
    fn try_pick(&mut self) -> Option<Card> {
        if let Some(card) = self.cards.pop() {
            self.card_removed(card);
            Some(card)
        } else {
            None
        }
    }

    fn try_pick_first(&mut self, card: Card) -> Option<Card> {
        // TODO: Why does using reverse iterator yield incorrect results?
        let pos = self.cards.iter().position(|c| *c == card);

        if let Some(pos) = pos {
            self.cards.remove(pos);
            self.card_removed(card);
            Some(card)
        } else {
            None
        }
    }

    fn reshuffle(&mut self)
    {
        self.cards.clear();
        self.fill_cards();
        self.cards.shuffle(&mut self.rng);
        self.running_count = 0;
        self.needs_reshuffle = false;
    }

    fn force_true_count(&mut self, true_count: f32) {
        self.cards.clear();
        self.fill_cards();
        self.running_count = 0;

        let mut prev = 0.0;
        let mut prev_card = None;

        if true_count > 0.0 {
            while self.true_count() < true_count {
                prev = self.true_count();
                prev_card = Some(self.remove_low_card());
            }
        } else if true_count < 0.0 {
            while self.true_count() > true_count {
                prev = self.true_count();
                prev_card = Some(self.remove_high_card());
            }
        }

        if (true_count - prev).abs() < (true_count - self.true_count()).abs() {
            if true_count > 0.0 {
                if let Some(card) = prev_card {
                    self.cards.push(card);
                    self.running_count -= 1;
                }
            } else if true_count < 0.0 {
                if let Some(card) = prev_card {
                    self.cards.push(card);
                    self.running_count += 1;
                }
            }
        }

        self.cards.shuffle(&mut self.rng);
        self.needs_reshuffle = false;
    }

    fn adjust_running_count(&mut self, rel_rc: i32) {
        if rel_rc > 0 {
            for _ in 0..rel_rc {
                self.remove_low_card();
            }
        } else if rel_rc < 0 {
            for _ in 0..-rel_rc {
                self.remove_high_card();
            }
        }
    }

    fn needs_reshuffle(&self) -> bool {
        self.needs_reshuffle
    }

    fn running_count(&self) -> i32 {
        self.running_count
    }

    fn true_count(&self) -> f32 {
        self.running_count as f32 / (self.cards.len() as f32 / 52.0)
    }
}

impl fmt::Display for StandardShoe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;

        for (i, card) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", card)?;
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use crate::card::Card;
    use crate::shoe::standard_shoe::StandardShoe;
    use crate::shoe::CardShoe;
    use crate::test_utils::assert_f64_eq;

    #[test]
    fn it_creates_a_one_deck_shoe() {
        let shoe = StandardShoe::non_shuffled(1, 52);

        assert_eq!(shoe.cards.len(), 52);

        for card in 1..=9 {
            assert_eq!(shoe.cards.iter().filter(|&&c| c == Card(card))
                           .count(), 4);
        }

        assert_eq!(shoe.cards.iter().filter(|&&c| c == Card(10)).count(), 16);
    }

    #[test]
    fn it_tries_to_pick_a_specific_card() {
        let mut shoe = StandardShoe::shuffled(1, 52);

        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), None);

        for card in 1..=9 {
            let expected_count = if card == 5 { 0 } else { 4 };
            assert_eq!(shoe.cards.iter().filter(|&&c| c == Card(card))
                           .count(), expected_count);
        }

        assert_eq!(shoe.cards.iter().filter(|&&c| c == Card(10)).count(), 16);
    }

    #[test]
    fn it_can_be_reshuffled() {
        let mut shoe = StandardShoe::shuffled(1, 52);

        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), None);

        shoe.reshuffle();

        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), Some(Card(5)));
        assert_eq!(shoe.try_pick_first(Card(5)), None);
    }

    #[test]
    fn it_picks_a_card() {
        let mut shoe = StandardShoe::non_shuffled(1, 52);

        assert_eq!(shoe.try_pick(), Some(Card(10)));
        assert_eq!(shoe.try_pick(), Some(Card(10)));
        assert_eq!(shoe.try_pick(), Some(Card(10)));
        assert_eq!(shoe.try_pick(), Some(Card(10)));
        assert_eq!(shoe.try_pick(), Some(Card(9)));
        assert_eq!(shoe.try_pick(), Some(Card(8)));
    }

    #[test]
    fn it_returns_the_running_count() {
        let mut shoe = StandardShoe::shuffled(4, 208);

        assert_eq!(shoe.running_count(), 0);
        shoe.try_pick_first(Card(8)).unwrap();
        assert_eq!(shoe.running_count(), 0);
        shoe.try_pick_first(Card(3)).unwrap();
        assert_eq!(shoe.running_count(), 1);
        shoe.try_pick_first(Card(6)).unwrap();
        shoe.try_pick_first(Card(2)).unwrap();
        assert_eq!(shoe.running_count(), 3);
        shoe.try_pick_first(Card(7)).unwrap();
        shoe.try_pick_first(Card(8)).unwrap();
        shoe.try_pick_first(Card(9)).unwrap();
        assert_eq!(shoe.running_count(), 3);
        shoe.try_pick_first(Card(10)).unwrap();
        assert_eq!(shoe.running_count(), 2);
        shoe.try_pick_first(Card(1)).unwrap();
        assert_eq!(shoe.running_count(), 1);
    }

    #[test]
    fn it_needs_reshuffling_because_of_penetration() {
        let mut shoe = StandardShoe::shuffled(1, 3);

        assert!(!shoe.needs_reshuffle());
        shoe.pick();
        shoe.pick();
        assert!(!shoe.needs_reshuffle());
        shoe.pick();
        assert!(shoe.needs_reshuffle());
        shoe.pick();
        assert!(shoe.needs_reshuffle());
    }

    #[test]
    fn it_forces_a_specific_true_count() {
        let mut shoe = StandardShoe::non_shuffled(2, 104);

        shoe.force_true_count(3.0);

        assert_eq!(shoe.running_count(), 6);
        assert_f64_eq(shoe.true_count() as f64, 3.184, 0.001);

        let mut shoe = StandardShoe::non_shuffled(2, 104);

        shoe.force_true_count(-5.0);

        assert_eq!(shoe.running_count(), -9);
        assert_f64_eq(shoe.true_count() as f64, -4.926, 0.001);
    }

    #[test]
    fn it_removes_all_high_cards_single_deck() {
        let mut shoe = StandardShoe::non_shuffled(1, 52);

        shoe.force_true_count(-32.5);

        assert_eq!(shoe.running_count(), -20);
        assert_eq!(shoe.true_count(), -32.5);
        assert_eq!(shoe.cards.iter()
                       .filter(|&&c| c == Card(10) || c == Card(1))
                       .count(), 0);
    }

    #[test]
    #[should_panic(expected = "Not enough high cards to reach desired true count")]
    fn it_panics_trying_to_remove_too_much_high_cards() {
        let mut shoe = StandardShoe::non_shuffled(1, 52);
        shoe.force_true_count(-33.0);
    }

    #[test]
    fn it_removes_low_high_cards_single_deck() {
        let mut shoe = StandardShoe::non_shuffled(1, 52);

        shoe.force_true_count(32.5);

        assert_eq!(shoe.running_count(), 20);
        assert_eq!(shoe.true_count(), 32.5);
        assert_eq!(shoe.cards.iter()
                       .filter(|&&c| c.0 >= 2 && c.0 <= 6)
                       .count(), 0);
    }

    #[test]
    #[should_panic(expected = "Not enough low cards to reach desired true count")]
    fn it_panics_trying_to_remove_too_much_low_cards() {
        let mut shoe = StandardShoe::non_shuffled(1, 52);
        shoe.force_true_count(33.0);
    }
}
