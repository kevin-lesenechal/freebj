use std::collections::VecDeque;

use crate::game_rules::GameRules;
use crate::strategy::{Strategy, Decision};
use crate::shoe::CardShoe;
use crate::round::Round;
use crate::betting::BettingStrategy;
use crate::card::Card;

pub struct RoundFactory<'a>
{
    rules: &'a GameRules,
    strategy: &'a (dyn Strategy + Sync),
    betting_strategy: &'a (dyn BettingStrategy + Sync),
    num_players: u8,
    holecarding: bool,
    override_action: Option<Decision>,
    surrender_override: Option<bool>,
    start_cards: VecDeque<Card>,
    dealer_cards: VecDeque<Card>,
}

impl<'a> RoundFactory<'a>
{
    pub fn new(rules: &'a GameRules,
               strategy: &'a (dyn Strategy + Sync),
               betting_strategy: &'a (dyn BettingStrategy + Sync),
               num_players: u8,
               holecarding: bool,
               override_action: Option<Decision>,
               surrender_override: Option<bool>,
               start_cards: VecDeque<Card>,
               dealer_cards: VecDeque<Card>) -> RoundFactory<'a> {
        RoundFactory {
            rules,
            strategy,
            betting_strategy,
            num_players,
            holecarding,
            override_action,
            surrender_override,
            start_cards,
            dealer_cards,
        }
    }

    pub fn make(&self, shoe: &'a mut dyn CardShoe) -> Round {
        Round::new(
            self.rules,
            self.strategy,
            self.betting_strategy,
            shoe,
            self.num_players,
            self.holecarding,
            self.override_action,
            self.surrender_override,
            &self.start_cards,
            &self.dealer_cards,
        )
    }
}
