use std::collections::{VecDeque, vec_deque};
use std::fmt::{Debug, Formatter};
use std::process::exit;
use arrayvec::ArrayVec;

use crate::game_rules::GameRules;
use crate::hand::Hand;
use crate::strategy::{Strategy, GameContext, Decision};
use crate::card::Card;
use crate::shoe::CardShoe;
use crate::game_rules::GameType::{Ahc, Enhc};
use crate::game_rules::SurrenderPolicy::{EarlySurrender, LateSurrender};
use crate::game_rules::Soft17::H17;
use crate::hand_stats::HandStats;
use crate::hand_logic::{hand_result, may_double};
use crate::betting::BettingStrategy;

pub struct Round<'a>
{
    rules:      &'a GameRules,
    context:    GameContext<'a>,
    strategy:   &'a dyn Strategy,
    betting_strategy: &'a dyn BettingStrategy,
    shoe:       &'a mut dyn CardShoe,
    dealer:     Hand,
    hands:      ArrayVec<[Hand; 32]>,
    hands_per_player: [u8; 7],
    start_cards: &'a VecDeque<Card>,
    dealer_cards: vec_deque::Iter<'a, Card>,
    holecarding: bool,
    override_action: Option<Decision>,
    surrender_override: Option<bool>,
}

#[derive(Debug)]
pub struct RoundResult {
    pub player_results: [f64; 7],
    pub hand_stats: HandStats,
}

impl<'a> Round<'a> {
    pub fn new(
        rules: &'a GameRules,
        strategy: &'a dyn Strategy,
        betting_strategy: &'a dyn BettingStrategy,
        shoe: &'a mut dyn CardShoe,
        num_players: u8,
        holecarding: bool,
        override_action: Option<Decision>,
        surrender_override: Option<bool>,
        start_cards: &'a VecDeque<Card>,
        dealer_cards: &'a VecDeque<Card>,
    ) -> Self {
        assert!(num_players > 0 && num_players < 8);

        let mut hands = ArrayVec::new();
        for _ in 0..num_players {
            hands.push(Hand::new());
        }

        assert!(!(surrender_override == Some(true)
                  && override_action.is_some()));

        Self {
            rules,
            context: GameContext {
                rules,
                may_split: false,
                may_double: false,
                true_count: 0.0,
                holecard: None,
            },
            strategy,
            betting_strategy,
            shoe,
            dealer: Hand::new(),
            hands,
            hands_per_player: [1; 7],
            start_cards,
            dealer_cards: dealer_cards.iter(),
            holecarding,
            override_action,
            surrender_override,
        }
    }

    pub fn run(mut self) -> (Self, RoundResult) {
        self.context.true_count = self.shoe.true_count();

        for hand in self.hands.iter_mut() {
            hand.bet = self.betting_strategy.place_bet(self.context.true_count);

            if self.start_cards.is_empty() {
                hand.add(self.shoe.pick());
            } else {
                hand.add(self.shoe.pick_first(self.start_cards[0]));
            }
        }

        self.dealer_pick();

        for hand in self.hands.iter_mut() {
            if self.start_cards.len() > 1 {
                for &card in self.start_cards.iter().skip(1) {
                    hand.add(self.shoe.pick_first(card));
                }
            } else {
                hand.add(self.shoe.pick());
            }
        }

        if self.rules.game_type == Ahc {
            self.dealer_pick();
            if self.holecarding {
                self.context.holecard = Some(self.dealer[1]);
            }
        }

        if self.rules.surrender == EarlySurrender {
            self.check_surrender();
        }

        if self.dealer[0] == Card(1) {
            for hand in self.hands.iter_mut() {
                if hand.is_surrendered() {
                    continue;
                }

                if self.strategy.take_insurance(&self.context, hand) {
                    hand.insure();
                }
            }
        }

        if self.rules.game_type == Enhc
           || (self.rules.game_type == Ahc && !self.dealer.is_bj()) {
            if self.rules.surrender == LateSurrender {
                self.check_surrender();
            }

            for i in 0..self.hands.len() {
                if !self.hands[i].is_surrendered() {
                    self.do_player_turn(i);
                }
            }
        }

        while self.dealer.value() < 17
              || (self.rules.soft17 == H17
                  && self.dealer.is_soft()
                  && self.dealer.value() == 17) {
            self.dealer_pick();
        }

        if self.shoe.needs_reshuffle() {
            self.shoe.reshuffle();
        }

        let mut player_results = [0.0; 7];
        let mut hand_stats = HandStats::default();

        for hand in self.hands.iter() {
            let (outcome, hand_result) = hand_result(hand, &self.dealer);
            player_results[hand.id as usize] += hand_result * hand.bet;
            hand_stats.update(hand, outcome);
        }

        (
            self,
            RoundResult {
                player_results,
                hand_stats,
            },
        )
    }

    fn dealer_pick(&mut self) {
        let card = match self.dealer_cards.next() {
            Some(&card) => self.shoe.pick_first(card),
            None => self.shoe.pick(),
        };
        self.dealer.add(card);
    }

    fn check_surrender(&mut self) {
        if self.surrender_override == Some(false) {
            return;
        }

        if self.surrender_override == Some(true) {
            for hand in self.hands.iter_mut() {
                hand.surrender();
            }
        } else {
            for hand in self.hands.iter_mut() {
                if !hand.is_surrendered() && self.strategy.surrender(
                    &self.context,
                    self.dealer[0],
                    hand,
                    self.rules.surrender == EarlySurrender,
                ) {
                    hand.surrender();
                }
            }
        }
    }

    fn do_player_turn(&mut self, i: usize) {
        loop {
            let hand = &self.hands[i];
            assert!(!hand.is_busted());
            let hands_count = self.hands_per_player[hand.id as usize] as u32;
            self.context.may_split = hands_count < self.rules.max_splits
                                     && hand.count() == 2
                                     && hand[0] == hand[1];
            self.context.may_double = may_double(
                self.rules.double_down,
                self.rules.das,
                hand
            );
            self.context.true_count = self.shoe.true_count();

            let decision = if let Some(action) = self.override_action {
                if action == Decision::Split && !self.context.may_split {
                    eprintln!("Unable to split");
                    exit(2);
                }
                // FIXME: does not work with multiple players
                self.override_action = None;

                action
            } else {
                self.strategy.player_turn(&self.context, self.dealer[0], hand)
            };

            let hand = &mut self.hands[i];

            match decision {
                Decision::Stand => {
                    return;
                },
                Decision::Hit => {
                    hand.add(self.shoe.pick());
                    if hand.is_busted() {
                        return;
                    }
                },
                Decision::Double => {
                    assert!(self.context.may_double,
                            "Doubling down is forbidden");
                    hand.add(self.shoe.pick());
                    hand.double_down();
                    return;
                },
                Decision::Split => {
                    assert!(self.context.may_split,
                            "Splitting is forbidden");
                    let id = hand.id;
                    let bet = hand.bet;

                    self.hands_per_player[id as usize] += 1;
                    let common = hand[0];
                    self.hands[i] = Hand::from(&[common, self.shoe.pick()][..]);
                    self.hands[i].bet = bet;
                    self.hands[i].split();
                    let mut new_hand = Hand::from(&[common, self.shoe.pick()][..]);
                    new_hand.id = id;
                    new_hand.bet = bet;
                    new_hand.split();
                    self.hands.push(new_hand);

                    if self.rules.play_ace_pairs || common != Card(1) {
                        let next = self.hands.len() - 1;
                        self.do_player_turn(i);
                        self.do_player_turn(next);
                    }
                    return;
                },
            }
        }
    }
}

impl Debug for Round<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for hand in self.hands.iter() {
            write!(f, "({}) {}\n", hand.id, hand)?;
        }

        write!(f, "Vs. {}", self.dealer)
    }
}

#[cfg(test)]
mod tests {
    use crate::round::Round;
    use crate::test_utils::{QueuedStrategy, make_rules};
    use crate::strategy::Decision::*;
    use crate::strategy::Decision;
    use crate::hand_stats::HandStats;
    use crate::betting::FixedBet;
    use crate::test_utils::options::*;
    use std::collections::VecDeque;
    use crate::shoe::queued_shoe::QueuedShoe;

    #[test]
    fn it_wins_a_hand() {
        //           Tt Wo Lo Pu Bu BJ Db Sp In Su
        test_result(&[10, 7, 9, 10], AHC|H17, &[Stand], 10.0,
                    (1, 1, 0, 0, 0, 0, 0, 0, 0, 0));
        test_result(&[6, 7, 1, 10, 6, 5], AHC|S17, &[Hit, Hit,Stand], 10.0,
                    (1, 1, 0, 0, 0, 0, 0, 0, 0, 0));
        test_result(&[10, 6, 8, 1], AHC|S17, &[Stand], 10.0,
                    (1, 1, 0, 0, 0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn it_loses_a_hand() {
        //                 Tt Wo Lo Pu Bu BJ Db Sp In Su
        test_result(&[10, 6, 8, 1, 9, 4], AHC|H17, &[Stand],
                    -10.0, (1, 0, 1, 0, 0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn it_busts_a_hand() {
        test_result(&[5, 8, 6, 2, 2, 3, 5, 1, 6, 8], AHC|S17, &[Hit, Hit, Hit, Hit],
                    -10.0, (1, 0, 1, 0, 1, 0, 0, 0, 0, 0));
        test_result(&[5, 8, 6, 2, 3, 5, 1, 2, 6, 5], AHC|S17, &[Hit, Hit, Hit, Hit],
                    -10.0, (1, 0, 1, 0, 1, 0, 0, 0, 0, 0));
    }

    #[test]
    fn it_doubles_down() {
        test_result(&[6, 7, 5, 10, 9], AHC|S17, &[Double], 20.0,
                    (1, 1, 0, 0, 0, 0, 1, 0, 0, 0));
        test_result(&[6, 7, 5, 10, 6], AHC|S17, &[Double], 0.0,
                    (1, 0, 0, 1, 0, 0, 1, 0, 0, 0));
        test_result(&[6, 7, 5, 10, 2], AHC|S17, &[Double], -20.0,
                    (1, 0, 1, 0, 0, 0, 1, 0, 0, 0));
    }

    #[test]
    fn it_insures() {
        //           Tt Wo Lo Pu Bu BJ Db Sp In Su
        test_result(&[8, 10, 10, 1], AHC|S17|INSURE, &[], -10.0,
                    (1, 0, 1, 0, 0, 0, 0, 0, 0, 0));
        test_result(&[8, 1, 10, 10], AHC|S17|INSURE, &[], 0.0,
                    (1, 0, 1, 0, 0, 0, 0, 0, 1, 0));
        test_result(&[1, 1, 10, 10], AHC|S17|INSURE, &[], 10.0,
                    (1, 0, 0, 1, 0, 1, 0, 0, 1, 0));
        test_result(&[1, 1, 10, 9], AHC|S17|INSURE, &[Stand], 10.0,
                    (1, 1, 0, 0, 0, 1, 0, 0, 1, 0));
        test_result(&[10, 1, 10, 9], AHC|S17|INSURE, &[Stand], -5.0,
                    (1, 0, 0, 1, 0, 0, 0, 0, 1, 0));
        test_result(&[10, 1, 9, 9], AHC|S17|INSURE, &[Stand], -15.0,
                    (1, 0, 1, 0, 0, 0, 0, 0, 1, 0));
        //           Tt Wo Lo Pu Bu BJ Db Sp In Su
    }

    #[test]
    fn it_split_pairs() {
        //                 Tt Wo Lo Pu Bu BJ Db Sp In Su
        test_result(&[8, 10, 8, 9, 10, 10], AHC|S17, &[Split, Stand, Stand],
                    -20.0, (2, 0, 2, 0, 0, 0, 0, 2, 0, 0));
        test_result(&[8, 10, 8, 7, 10, 5], AHC|S17, &[Split, Stand, Stand],
                     0.0, (2, 1, 1, 0, 0, 0, 0, 2, 0, 0));
        test_result(&[1, 10, 1, 7, 10, 10], AHC|S17, &[Split],
                     20.0, (2, 2, 0, 0, 0, 0, 0, 2, 0, 0));
        test_result(&[1, 10, 1, 7, 4, 5], AHC|S17, &[Split],
                    -20.0, (2, 0, 2, 0, 0, 0, 0, 2, 0, 0));
        test_result(&[1, 10, 1, 7, 4, 5, 4, 2], AHC|S17|HAA,
                    &[Split, Hit, Stand, Hit, Stand],
                     20.0, (2, 2, 0, 0, 0, 0, 0, 2, 0, 0));
        test_result(&[2, 7, 2, 10, 2, 9, 9, 8, 2, 2, 8, 2, 10, 10], AHC|S17,
                    &[Split, Split, Hit, Hit, Hit, Hit, Hit, Hit, Stand],
                    -10.0, (3, 1, 2, 0, 2, 0, 0, 3, 0, 0));
        //                 Tt Wo Lo Pu Bu BJ Db Sp In Su
    }

    #[test]
    #[should_panic(expected = "Doubling down is forbidden")]
    fn it_panics_when_doubling_split_pairs_with_no_das() {
        let rules = make_rules(AHC|S17);
        let start_cards = VecDeque::new();
        let strategy = QueuedStrategy::new(&[Split, Double, Double], false, false);
        let mut shoe = QueuedShoe::from_ints(&[8, 6, 8, 10, 3, 1]);
        Round::new(&rules, &strategy, &FixedBet(1.0), &mut shoe,
                   1, false, None, None, &start_cards, &start_cards)
            .run();
    }

    #[test]
    #[should_panic(expected = "Splitting is forbidden")]
    fn it_panics_when_splitting_non_pairs() {
        let rules = make_rules(AHC|S17);
        let start_cards = VecDeque::new();
        let strategy = QueuedStrategy::new(&[Split], false, false);
        let mut shoe = QueuedShoe::from_ints(&[8, 6, 7, 10]);
        Round::new(&rules, &strategy, &FixedBet(1.0), &mut shoe,
                   1, false, None, None, &start_cards, &start_cards)
            .run();
    }

    #[test]
    #[should_panic(expected = "Splitting is forbidden")]
    fn it_panics_when_splitting_above_max() {
        let mut rules = make_rules(AHC|S17);
        rules.max_splits = 2;
        let start_cards = VecDeque::new();
        let strategy = QueuedStrategy::new(&[Split, Split], false, false);
        let mut shoe = QueuedShoe::from_ints(&[8, 6, 8, 10, 8, 7]);
        Round::new(&rules, &strategy, &FixedBet(1.0), &mut shoe,
                   1, false, None, None, &start_cards, &start_cards)
            .run();
    }

    #[test]
    #[should_panic(expected = "Splitting is forbidden")]
    fn it_panics_when_splitting_with_three_cards() {
        let rules = make_rules(AHC|S17);
        let start_cards = VecDeque::new();
        let strategy = QueuedStrategy::new(&[Hit, Split], false, false);
        let mut shoe = QueuedShoe::from_ints(&[4, 6, 4, 10, 4]);
        Round::new(&rules, &strategy, &FixedBet(1.0), &mut shoe,
                   1, false, None, None, &start_cards, &start_cards)
            .run();
    }

    const INSURE: u32    = 1 << 3;
    const SURRENDER: u32 = 1 << 4;

    fn test_result(cards: &[u8],
                   opts: u32,
                   decisions: &[Decision],
                   expected_result: f64,
                   expected_stats: (u64, u64, u64, u64, u64, u64, u64, u64, u64, u64)) {
        let rules = make_rules(opts);
        let start_cards = VecDeque::new();
        let strategy = QueuedStrategy::new(
            decisions,
            opts & INSURE > 0,
            opts & SURRENDER > 0,
        );
        let betting = FixedBet(10.0);
        let mut shoe = QueuedShoe::from_ints(cards);
        let round = Round::new(&rules, &strategy, &betting, &mut shoe,
                               1, false, None, None,
                               &start_cards, &start_cards);

        let (_, result) = round.run();

        assert_eq!(result.player_results,
                   [expected_result, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(result.hand_stats, HandStats::from(expected_stats),
                   "{:?}", cards);
        assert!(shoe.is_empty());
        assert!(strategy.is_empty(), "Not all decisions were taken");
    }

    // TODO: test surrender override
}
