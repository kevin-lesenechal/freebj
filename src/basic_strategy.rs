use crate::strategy::{Strategy, GameContext, Decision};
use crate::hand::Hand;
use crate::card::Card;
use crate::strategy::Decision::*;
use crate::game_rules::GameType::Ahc;
use crate::game_rules::Soft17::{H17, S17};
use crate::deviation::{Deviation, DeviationTable};

static HARD_TABLE: [&[u8; 11]; 17] = [
    // A23456789J
    b" ==========", // 20
    b" ==========", // 19
    b" ==========", // 18
    b" u=========", // 17
    b" S=====++SS", // 16
    b" U=====+++S", // 15
    b" E=====+++E", // 14
    b" E=====++++", // 13
    b" E++===++++", // 12
    // A23456789J
    b" &DDDDDDDD?", // 11
    b" +DDDDDDDD+", // 10
    b" ++DDDD++++", // 9
    b" ++++++++++", // 8
    b" ++++++++++", // 7
    b" ++++++++++", // 6
    b" ++++++++++", // 5
    b" ++++++++++", // 4
    // A23456789J
];

static SOFT_TABLE: [&[u8; 11]; 10] = [
    // A23456789J
    b" ==========", // 10
    b" ==========", // 9
    b" =====h====", // 8
    b" +hdddd==++", // 7
    b" ++DDDD++++", // 6
    b" +++DDD++++", // 5
    b" +++DDD++++", // 4
    b" ++++DD++++", // 3
    b" ++++DD++++", // 2
    b" +++++D++++", // A
    // A23456789J
];

static PAIRS_TABLE: [&[u8; 11]; 10] = [
    // A23456789J
    b"           ", // T/T
    b"  VVVVV VV ", // 9/9
    b" @VVVVVVVV?", // 8/8
    b"  VVVVVV   ", // 7/7
    b"  *VVVV    ", // 6/6
    b"           ", // 5/5
    b"     **    ", // 4/4
    b"  **VVVV   ", // 3/3
    b"  **VVVV   ", // 2/2
    b" ?VVVVVVVVV", // A/A
    // A23456789J
];

static DEFAULT_HARD_DEVIATIONS: [[u8; 40]; 17] = [
    //  A | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
    *b"                                        ", // 20
    *b"                                        ", // 19
    *b"                                        ", // 18
    *b"                                        ", // 17
    *b"                                 +4=>+1=", // 16
    *b"                                     +4=", // 15
    *b"                                        ", // 14
    *b"                                        ", // 13
    *b"                                        ", // 12
    //  A | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
    *b"                                        ", // 11
    *b"                                        ", // 10
    *b"                         +3D            ", // 9
    *b"                     +2D                ", // 8
    *b"                                        ", // 7
    *b"                                        ", // 6
    *b"                                        ", // 5
    *b"                                        ", // 4
    //  A | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
];

pub struct BasicStrategy {
    hilo: bool,
    deviations: bool,
    dev_hard_table: Box<[[u8; 40]; 17]>,
    dev_soft_table: Box<[[u8; 40]; 10]>,
    dev_pair_table: Box<[[u8; 40]; 10]>,
}

impl BasicStrategy {
    pub fn new(hilo: bool) -> BasicStrategy {
        BasicStrategy {
            hilo,
            deviations: false,
            dev_hard_table: Box::new([[b' '; 40]; 17]),
            dev_soft_table: Box::new([[b' '; 40]; 10]),
            dev_pair_table: Box::new([[b' '; 40]; 10]),
        }
    }

    pub fn set_default_deviations(&mut self) {
        self.dev_hard_table = Box::new(DEFAULT_HARD_DEVIATIONS);
        self.deviations = true;
    }

    pub fn add_deviation(&mut self, deviation: Deviation) {
        let ov_str = deviation.action.to_string();
        assert_eq!(ov_str.len(), 4);

        let slice: &mut [u8] = match deviation.table {
            DeviationTable::HardTable => {
                &mut self.dev_hard_table[deviation.row as usize]
            },
            DeviationTable::SoftTable => {
                &mut self.dev_soft_table[deviation.row as usize]
            },
            DeviationTable::PairTable => {
                &mut self.dev_pair_table[deviation.row as usize]
            },
        };

        let index = (deviation.dealer as usize - 1) << 2;
        let slice = &mut slice[index..index + 4];

        slice.copy_from_slice(ov_str.as_bytes());

        self.deviations = true;
    }

    fn basic_strategy(&self,
                      game: &GameContext,
                      dealer: Card,
                      me: &Hand) -> u8 {
        if me.value() == 21 {
            return b'=';
        }

        if game.may_split && me.count() == 2 && me[0] == me[1] {
            let ch = PAIRS_TABLE[10 - me[0].0 as usize][dealer.0 as usize];
            let ahc = game.rules.game_type == Ahc;

            if ch == b'V'
               || (ch == b'*' && game.rules.das)
               || (ch == b'?' && ahc)
               || (ch == b'@' && ahc && game.rules.soft17 == S17) {
                return b'V';
            }
        }

        if me.is_soft() {
            let soft_sum = me.iter().map(|c| c.0 as usize).sum::<usize>() - 1;
            SOFT_TABLE[10 - soft_sum][dealer.0 as usize]
        } else {
            HARD_TABLE[20 - me.value() as usize][dealer.0 as usize]
        }
    }

    fn holecarding_strategy(&self,
                            _game: &GameContext,
                            _d1: Card,
                            _d2: Card,
                            _me: &Hand) -> u8 {
        unimplemented!()
    }

    fn apply_deviations(&self,
                        decision: &mut u8,
                        game: &GameContext,
                        dealer: Card,
                        me: &Hand) {
        let tc = game.true_count.round() as i8;
        let val = me.value() as usize;

        if val == 21 {
            return;
        }

        let val = me.value() as usize;
        let d_index = (dealer.0 as usize - 1) << 2;

        if game.may_split && me.count() == 2 && me[0] == me[1] {
            let dev = &self.dev_pair_table[10 - me[0].0 as usize]
                [d_index..d_index + 4];
            if let Some(action) = self.try_deviate(dev, tc) {
                *decision = action;
                return;
            }
        }

        let dev;

        if me.is_soft() {
            let soft_sum = me.iter().map(|c| c.0 as usize).sum::<usize>() - 1;
            dev = &self.dev_soft_table[10 - soft_sum][d_index..d_index + 4];
        } else {
            dev = &self.dev_hard_table[20 - val][d_index..d_index + 4];
        }

        if let Some(action) = self.try_deviate(dev, tc) {
            *decision = action;
            return;
        }
    }

    fn try_deviate(&self, dev: &[u8], tc: i8) -> Option<u8> {
        if dev[0] == b' ' {
            return None;
        }

        debug_assert!(dev[0] == b'>' || dev[0] == b'<');
        debug_assert!(dev[1] == b'+' || dev[1] == b'-');
        debug_assert!(dev[2] >= b'0' && dev[2] <= b'9');

        let mut tc_trig = (dev[2] - b'0') as i8;
        if dev[1] == b'-' {
            tc_trig = -tc_trig;
        }

        if (dev[0] == b'>' && tc >= tc_trig)
            || (dev[0] == b'<' && tc <= tc_trig) {
            Some(dev[3])
        } else {
            None
        }
    }
}

impl Strategy for BasicStrategy {
    fn player_turn(&self,
                   game: &GameContext,
                   dealer: Card,
                   me: &Hand) -> Decision {
        let mut decision = if let Some(holecard) = game.holecard {
            self.holecarding_strategy(game, dealer, holecard, me)
        } else {
            self.basic_strategy(game, dealer, me)
        };

        if self.deviations {
            self.apply_deviations(&mut decision, game, dealer, me);
        }

        decision = match decision {
            b'?' if game.rules.game_type == Ahc => b'D',
            b'?' => b'+',
            b'&' if game.rules.game_type == Ahc
                   && game.rules.soft17 == H17 => b'D',
            b'&' => b'+',
            b'h' if game.rules.soft17 == H17 => b'd',
            b'h' => b'=',
            _ => decision,
        };

        decision = match decision {
            b'D' if !game.may_double => b'+',
            b'd' if !game.may_double => b'=',
            _ => decision,
        };

        match decision {
            b'+' | b'S' | b'E' | b'U' => Hit,
            b'=' | b's' | b'e' | b'u' => Stand,
            b'D' | b'd' => Double,
            b'V' => Split,
            _ => panic!("Unknown basic strategy decision '{}'", decision),
        }
    }

    fn surrender(&self,
                 game: &GameContext,
                 dealer: Card,
                 me: &Hand,
                 is_early: bool) -> bool {
        if me.is_soft() {
            return false;
        }

        let decision = if let Some(holecard) = game.holecard {
            self.holecarding_strategy(game, dealer, holecard, me)
        } else {
            self.basic_strategy(game, dealer, me)
        };

        match decision {
            b'S' | b's' => true,
            b'E' | b'e' if is_early => true,
            b'U' | b'u' if is_early || game.rules.soft17 == H17 => true,
            _ => false,
        }
    }

    fn take_insurance(&self, game: &GameContext, _me: &Hand) -> bool {
        if let Some(holecard) = game.holecard {
            holecard == Card(10)
        } else if !self.hilo {
            false
        } else {
            game.true_count >= 3.0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::strategy::{Decision, GameContext, Strategy};
    use crate::card::Card;
    use crate::basic_strategy::BasicStrategy;
    use crate::game_rules::GameRules;
    use crate::strategy::Decision::*;
    use crate::hand::Hand;
    use crate::test_utils::make_rules;
    use crate::deviation::Deviation;
    use std::str::FromStr;

    #[test]
    fn it_plays_hard_hands() {
        test_decision(Stand,  &[3, 7, 10], 10, AHC|S17);

        test_decision(Stand,  &[10, 7],  7,  AHC|S17);

        test_decision(Stand,  &[10, 6],  6,  AHC|S17);
        test_decision(Hit,    &[10, 6],  7,  AHC|S17);

        test_decision(Hit,    &[10, 2],  2,  AHC|S17);
        test_decision(Hit,    &[10, 2],  3,  AHC|S17);
        test_decision(Stand,  &[10, 2],  4,  AHC|S17);
        test_decision(Stand,  &[10, 2],  6,  AHC|S17);
        test_decision(Hit,    &[10, 2],  10, AHC|S17);

        test_decision(Double, &[4, 7],   6,  AHC|S17);
        test_decision(Hit,    &[4, 7],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Double, &[4, 7],   10, AHC|S17);
        test_decision(Hit,    &[4, 7],   10, ENHC|S17);
        test_decision(Hit,    &[4, 7],   1,  ENHC|S17);
        test_decision(Hit,    &[4, 7],   1,  AHC|S17);
        test_decision(Double, &[4, 7],   1,  AHC|H17);

        test_decision(Double, &[4, 6],   6,  AHC|S17);
        test_decision(Hit,    &[4, 6],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Hit,    &[4, 6],   10, AHC|S17);
        test_decision(Hit,    &[4, 6],   1,  AHC|S17);

        test_decision(Hit,    &[4, 5],   2,  AHC|S17);
        test_decision(Double, &[4, 5],   3,  AHC|S17);
        test_decision(Double, &[4, 5],   6,  AHC|S17);
        test_decision(Hit,    &[4, 5],   7,  AHC|S17);

        test_decision(Hit,    &[5, 3],   6,  AHC|S17);

        test_decision(Hit,    &[2, 2],   2,  AHC|S17|NO_SPLIT);
        test_decision(Hit,    &[2, 2],   1,  AHC|S17|NO_SPLIT);
    }

    #[test]
    fn it_plays_soft_hands() {
        test_decision(Stand,  &[10, 1],  2,  AHC|S17);
        test_decision(Stand,  &[10, 1],  1,  AHC|S17);

        test_decision(Stand,  &[1, 9],   2,  AHC|S17);
        test_decision(Stand,  &[1, 9],   6,  AHC|S17);
        test_decision(Stand,  &[1, 9],   10, AHC|S17);
        test_decision(Stand,  &[1, 9],   1,  AHC|S17);

        test_decision(Stand,  &[1, 8],   6,  AHC|S17);
        test_decision(Double, &[1, 8],   6,  AHC|H17);
        test_decision(Stand,  &[1, 8],   6,  AHC|H17|NO_DOUBLE);
        test_decision(Stand,  &[1, 8],   10, AHC|S17);
        test_decision(Stand,  &[1, 8],   1,  AHC|S17);

        test_decision(Stand,  &[1, 7],   2,  AHC|S17);
        test_decision(Double, &[1, 7],   2,  AHC|H17);
        test_decision(Stand,  &[1, 7],   2,  AHC|H17|NO_DOUBLE);
        test_decision(Double, &[1, 7],   3,  AHC|S17);
        test_decision(Stand,  &[1, 7],   3,  AHC|S17|NO_DOUBLE);
        test_decision(Double, &[1, 7],   6,  AHC|S17);
        test_decision(Stand,  &[1, 7],   7,  AHC|S17);
        test_decision(Stand,  &[1, 7],   8,  AHC|S17);
        test_decision(Hit,    &[1, 7],   9,  AHC|S17);
        test_decision(Hit,    &[1, 7],   10, AHC|S17);
        test_decision(Hit,    &[1, 7],   1,  AHC|S17);

        test_decision(Hit,    &[1, 6],   2,  AHC|S17);
        test_decision(Double, &[1, 6],   3,  AHC|S17);
        test_decision(Double, &[1, 6],   6,  AHC|S17);
        test_decision(Hit,    &[1, 6],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Hit,    &[1, 6],   7,  AHC|S17);
        test_decision(Hit,    &[1, 6],   10, AHC|S17);

        test_decision(Hit,    &[1, 5],   3,  AHC|S17);
        test_decision(Double, &[1, 5],   4,  AHC|S17);
        test_decision(Double, &[1, 5],   6,  AHC|S17);
        test_decision(Hit,    &[1, 5],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Hit,    &[1, 5],   7,  AHC|S17);
        test_decision(Hit,    &[1, 5],   10, AHC|S17);

        test_decision(Hit,    &[1, 4],   3,  AHC|S17);
        test_decision(Double, &[1, 4],   4,  AHC|S17);
        test_decision(Double, &[1, 4],   6,  AHC|S17);
        test_decision(Hit,    &[1, 4],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Hit,    &[1, 4],   7,  AHC|S17);
        test_decision(Hit,    &[1, 4],   10, AHC|S17);

        test_decision(Hit,    &[1, 3],   4,  AHC|S17);
        test_decision(Double, &[1, 3],   5,  AHC|S17);
        test_decision(Double, &[1, 3],   6,  AHC|S17);
        test_decision(Hit,    &[1, 3],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Hit,    &[1, 3],   7,  AHC|S17);
        test_decision(Hit,    &[1, 3],   10, AHC|S17);

        test_decision(Hit,    &[1, 2],   4,  AHC|S17);
        test_decision(Double, &[1, 2],   5,  AHC|S17);
        test_decision(Double, &[1, 2],   6,  AHC|S17);
        test_decision(Hit,    &[1, 2],   6,  AHC|S17|NO_DOUBLE);
        test_decision(Hit,    &[1, 2],   7,  AHC|S17);
        test_decision(Hit,    &[1, 2],   10, AHC|S17);
        test_decision(Hit,    &[1, 2],   1,  AHC|S17);
    }

    #[test]
    fn it_plays_pairs() {
        test_decision(Split,  &[1, 1],   2,  AHC|S17);
        test_decision(Split,  &[1, 1],   6,  AHC|S17);
        test_decision(Split,  &[1, 1],   7,  AHC|S17);
        test_decision(Split,  &[1, 1],   10, AHC|S17);
        test_decision(Split,  &[1, 1],   1,  AHC|S17);
        test_decision(Hit,    &[1, 1],   1,  ENHC|S17);

        test_decision(Stand,  &[10, 10], 2,  AHC|S17);
        test_decision(Stand,  &[10, 10], 6,  AHC|S17);
        test_decision(Stand,  &[10, 10], 7,  AHC|S17);
        test_decision(Stand,  &[10, 10], 10, AHC|S17);
        test_decision(Stand,  &[10, 10], 1,  AHC|S17);

        test_decision(Split,  &[9, 9],   2,  AHC|S17);
        test_decision(Split,  &[9, 9],   6,  AHC|S17);
        test_decision(Stand,  &[9, 9],   7,  AHC|S17);
        test_decision(Split,  &[9, 9],   8,  AHC|S17);
        test_decision(Split,  &[9, 9],   9,  AHC|S17);
        test_decision(Stand,  &[9, 9],   10, AHC|S17);
        test_decision(Stand,  &[9, 9],   1,  AHC|S17);

        test_decision(Split,  &[8, 8],   2,  AHC|S17);
        test_decision(Split,  &[8, 8],   6,  AHC|S17);
        test_decision(Split,  &[8, 8],   7,  AHC|S17);
        test_decision(Split,  &[8, 8],   10, AHC|S17);
        test_decision(Hit,    &[8, 8],   10, ENHC|S17);
        test_decision(Split,  &[8, 8],   1,  AHC|S17);
        test_decision(Hit,    &[8, 8],   1,  AHC|H17);
        test_decision(Hit,    &[8, 8],   1,  ENHC|S17);
        test_decision(Hit,    &[8, 8],   1,  ENHC|H17);

        test_decision(Split,  &[7, 7],   2,  AHC|S17);
        test_decision(Split,  &[7, 7],   6,  AHC|S17);
        test_decision(Split,  &[7, 7],   7,  AHC|S17);
        test_decision(Hit,    &[7, 7],   8,  AHC|S17);

        test_decision(Hit,    &[6, 6],   2,  AHC|S17);
        test_decision(Split,  &[6, 6],   2,  AHC|S17|DAS);
        test_decision(Split,  &[6, 6],   6,  AHC|S17);
        test_decision(Hit,    &[6, 6],   7,  AHC|S17);

        test_decision(Double, &[5, 5],   2,  AHC|S17);
        test_decision(Double, &[5, 5],   6,  AHC|S17);
        test_decision(Double, &[5, 5],   7,  AHC|S17);
        test_decision(Hit,    &[5, 5],   10, AHC|S17);
        test_decision(Hit,    &[5, 5],   1,  AHC|S17);

        test_decision(Hit,    &[4, 4],   2,  AHC|S17);
        test_decision(Hit,    &[4, 4],   4,  AHC|S17);
        test_decision(Hit,    &[4, 4],   5,  AHC|S17);
        test_decision(Split,  &[4, 4],   5,  AHC|S17|DAS);
        test_decision(Hit,    &[4, 4],   6,  AHC|S17);
        test_decision(Split,  &[4, 4],   6,  AHC|S17|DAS);
        test_decision(Hit,    &[4, 4],   7,  AHC|S17);

        test_decision(Hit,    &[3, 3],   2,  AHC|S17);
        test_decision(Split,  &[3, 3],   2,  AHC|S17|DAS);
        test_decision(Hit,    &[3, 3],   3,  AHC|S17);
        test_decision(Split,  &[3, 3],   3,  AHC|S17|DAS);
        test_decision(Split,  &[3, 3],   4,  AHC|S17);
        test_decision(Split,  &[3, 3],   7,  AHC|S17);
        test_decision(Hit,    &[3, 3],   8,  AHC|S17);

        test_decision(Hit,    &[2, 2],   2,  AHC|S17);
        test_decision(Split,  &[2, 2],   2,  AHC|S17|DAS);
        test_decision(Hit,    &[2, 2],   3,  AHC|S17);
        test_decision(Split,  &[2, 2],   3,  AHC|S17|DAS);
        test_decision(Split,  &[2, 2],   4,  AHC|S17);
        test_decision(Split,  &[2, 2],   7,  AHC|S17);
        test_decision(Hit,    &[2, 2],   8,  AHC|S17);
    }

    #[test]
    fn it_plays_surrenders() {
        test_surrender(false, &[10, 7],  1,  AHC|S17|LSURR);
        test_surrender(true,  &[10, 7],  1,  AHC|H17|LSURR);
        test_surrender(true,  &[10, 7],  1,  AHC|S17|ESURR);
        test_surrender(true,  &[10, 7],  1,  AHC|H17|ESURR);

        test_surrender(false, &[10, 5],  1,  AHC|S17|LSURR);
        test_surrender(true,  &[10, 5],  1,  AHC|H17|LSURR);
        test_surrender(true,  &[10, 5],  1,  AHC|S17|ESURR);
        test_surrender(true,  &[10, 5],  1,  AHC|H17|ESURR);
    }

    #[test]
    fn it_plays_specific_deviations() {
        let rules = make_rules(AHC|S17);
        let mut game = make_context(&rules, 0);

        let mut strat = BasicStrategy::new(true);
        strat.add_deviation(Deviation::from_str("20vs8:>+2D").unwrap());
        strat.add_deviation(Deviation::from_str("T/Tvs8:>+5V").unwrap());
        strat.add_deviation(Deviation::from_str("A5vs2:<-2D").unwrap());

        game.true_count = 0.0;
        assert_eq!(strat.player_turn(&game, Card(8),
                                     &make_player_hand(&[10, 10])),
                   Decision::Stand);

        game.true_count = 2.0;
        assert_eq!(strat.player_turn(&game, Card(8),
                                     &make_player_hand(&[10, 10])),
                   Decision::Double);

        game.true_count = 3.5;
        assert_eq!(strat.player_turn(&game, Card(8),
                                     &make_player_hand(&[10, 10])),
                   Decision::Double);

        game.true_count = 5.0;
        assert_eq!(strat.player_turn(&game, Card(8),
                                     &make_player_hand(&[10, 10])),
                   Decision::Split);

        game.true_count = 0.0;
        assert_eq!(strat.player_turn(&game, Card(2),
                                     &make_player_hand(&[1, 5])),
                   Decision::Hit);

        game.true_count = -2.0;
        assert_eq!(strat.player_turn(&game, Card(2),
                                     &make_player_hand(&[1, 5])),
                   Decision::Double);
        assert_eq!(strat.player_turn(&game, Card(3),
                                     &make_player_hand(&[1, 5])),
                   Decision::Hit);
    }

    const AHC: u32          = 0;
    const ENHC: u32         = 1 << 0;
    const S17: u32          = 0;
    const H17: u32          = 1 << 1;
    const DAS: u32          = 1 << 2;
    const NO_DOUBLE: u32    = 1 << 3;
    const NO_SPLIT: u32     = 1 << 4;
    const ESURR: u32        = 0;
    const LSURR: u32        = 1 << 5;

    fn make_context(rules: &GameRules, opts: u32) -> GameContext {
        GameContext {
            rules,
            may_split: opts & NO_SPLIT == 0,
            may_double: opts & NO_DOUBLE == 0,
            true_count: 0.0,
            holecard: None,
        }
    }

    fn make_player_hand(cards: &[u8]) -> Hand {
        let mut hand = Hand::new();

        for card in cards {
            hand.add(Card(*card));
        }

        hand
    }

    fn test_decision(expected: Decision,
                     hand: &[u8],
                     dealer: u8,
                     opts: u32) {
        let strategy = BasicStrategy::new(false);
        let rules = make_rules(opts);
        let decision = strategy.player_turn(
            &make_context(&rules, opts),
            Card(dealer),
            &make_player_hand(hand)
        );

        assert_eq!(decision, expected,
                   "{:?} vs {:?}, opts={:#b}", hand, dealer, opts);
    }

    fn test_surrender(expected: bool,
                      hand: &[u8],
                      dealer: u8,
                      opts: u32) {
        let strategy = BasicStrategy::new(false);
        let rules = make_rules(opts);
        let decision = strategy.surrender(
            &make_context(&rules, opts),
            Card(dealer),
            &make_player_hand(hand),
            opts & LSURR == 0
        );

        assert_eq!(decision, expected,
                   "{:?} vs {:?}, opts={:#b}", hand, dealer, opts);
    }
}
