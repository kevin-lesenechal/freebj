use std::fmt::Display;
use crate::card::Card;

pub mod standard_shoe;
pub mod file_shoe;
pub mod queued_shoe;

pub trait CardShoe: Display {
    fn try_pick(&mut self) -> Option<Card>;

    fn try_pick_first(&mut self, card: Card) -> Option<Card>;

    fn pick(&mut self) -> Card {
        self.try_pick().unwrap_or_else(|| {
            self.reshuffle();
            self.try_pick().expect("Couldn't pick any card after reshuffling")
        })
    }

    fn pick_first(&mut self, card: Card) -> Card {
        self.try_pick_first(card).unwrap_or_else(|| {
            self.reshuffle();
            self.try_pick_first(card)
                .expect("Couldn't find the card after reshuffling")
        })
    }

    fn reshuffle(&mut self);

    fn force_true_count(&mut self, true_count: f32);

    fn adjust_running_count(&mut self, rel_rc: i32);

    fn needs_reshuffle(&self) -> bool { false }

    fn running_count(&self) -> i32 { 0 }

    fn true_count(&self) -> f32 { 0.0 }
}
