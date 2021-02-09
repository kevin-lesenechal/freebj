use std::fmt;
use std::collections::VecDeque;

use crate::card::Card;
use crate::shoe::CardShoe;

pub struct QueuedShoe {
    cards: VecDeque<Card>,
}

impl QueuedShoe {
    pub fn new(cards: &[Card]) -> QueuedShoe {
        QueuedShoe {
            cards: cards.into_iter().map(|c| *c).collect(),
        }
    }

    pub fn from_ints(cards: &[u8]) -> QueuedShoe {
        QueuedShoe {
            cards: cards.into_iter().map(|&c| {
                assert!(c > 0 && c < 11);
                Card(c)
            }).collect()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl fmt::Display for QueuedShoe {
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

impl CardShoe for QueuedShoe {
    fn try_pick(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }

    fn try_pick_first(&mut self, card: Card) -> Option<Card> {
        self.cards.iter().position(|c| *c == card)
            .map(|i| self.cards.remove(i).unwrap())
    }

    fn pick(&mut self) -> Card {
        self.try_pick().expect("No cards left")
    }

    fn pick_first(&mut self, card: Card) -> Card {
        self.try_pick_first(card).expect("Couldn't find card")
    }

    fn reshuffle(&mut self) {
        unimplemented!("Cannot reshuffle a QueuedShoe");
    }

    fn force_true_count(&mut self, _true_count: f32) {
        unimplemented!("Cannot force true count to a QueuedShoe");
    }

    fn adjust_running_count(&mut self, _rel_rc: i32) {
        unimplemented!("Cannot adjust running count on a QueuedShoe")
    }
}

#[cfg(test)]
mod tests {
    use crate::shoe::queued_shoe::QueuedShoe;
    use crate::card::Card;
    use crate::shoe::CardShoe;

    #[test]
    fn it_picks_a_card_in_queued_shoe() {
        let mut shoe = QueuedShoe::from_ints(&[5, 6, 8][..]);

        assert_eq!(shoe.pick(), Card(5));
        assert_eq!(shoe.pick(), Card(6));
        assert!(!shoe.is_empty());
        assert_eq!(shoe.pick(), Card(8));
        assert!(shoe.is_empty());
    }

    #[test]
    #[should_panic(expected="No cards left")]
    fn it_panics_when_picking_too_many_cards() {
        let mut shoe = QueuedShoe::from_ints(&[5, 6, 8][..]);

        assert_eq!(shoe.pick(), Card(5));
        assert_eq!(shoe.pick(), Card(6));
        assert_eq!(shoe.pick(), Card(8));
        shoe.pick();
    }
}
