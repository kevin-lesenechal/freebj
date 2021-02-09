use crate::strategy::{Decision, Strategy, GameContext};
use crate::hand::Hand;
use crate::card::Card;
use std::collections::VecDeque;
use bitflags::_core::cell::RefCell;
use std::iter::FromIterator;
use crate::game_rules::{GameRules, GameType, Soft17};

pub struct QueuedStrategy {
    decisions: RefCell<VecDeque<Decision>>,
    take_insurance: bool,
    surrender: bool,
}

impl QueuedStrategy {
    pub fn new(decisions: &[Decision],
               take_insurance: bool,
               surrender: bool) -> QueuedStrategy {
        let vec = VecDeque::from_iter(decisions.iter().map(|&c| c));

        QueuedStrategy {
            decisions: RefCell::new(vec),
            take_insurance,
            surrender,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.decisions.borrow().is_empty()
    }
}

impl Strategy for QueuedStrategy {
    fn player_turn(&self,
                   _game: &GameContext,
                   _dealer: Card,
                   _me: &Hand) -> Decision {
        self.decisions.borrow_mut().pop_front()
            .expect("No more decisions in the queue")
    }

    fn surrender(&self,
                 _game: &GameContext,
                 _dealer: Card,
                 _me: &Hand,
                 _is_early: bool) -> bool {
        self.surrender
    }

    fn take_insurance(&self, _game: &GameContext, _me: &Hand) -> bool {
        self.take_insurance
    }
}

pub mod options {
    pub const AHC: u32      = 0;
    pub const ENHC: u32     = 1 << 0;
    pub const S17: u32      = 0;
    pub const H17: u32      = 1 << 1;
    pub const DAS: u32      = 1 << 2;
    pub const HAA: u32      = 1 << 3;
}

pub fn make_rules(opts: u32) -> GameRules {
    use options::*;

    GameRules {
        game_type: if opts & ENHC > 0 { GameType::Enhc } else { GameType::Ahc },
        soft17: if opts & H17 > 0 { Soft17::H17 } else { Soft17::S17 },
        das: opts & DAS > 0,
        play_ace_pairs: opts & HAA > 0,
        ..GameRules::default()
    }
}

pub fn assert_f64_eq(actual: f64, expected: f64, within: f64) {
    assert!((expected - actual).abs() < within,
            "Expected: {}\nActual:   {}\nWithin:   {}",
            expected, actual, within);
}
