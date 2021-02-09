use std::ops::{Add, AddAssign};
use serde::Serialize;
use crate::hand::Hand;
use crate::hand_logic::HandOutcome;

#[derive(Debug, PartialEq, Serialize)]
pub struct HandStats {
    /// Total number of hands a player played, this includes split hands,
    /// surrendered hands, and naturals.
    pub total:      u64,

    /// Total number of hands the player won, which includes naturals, busted
    /// dealer, and won insured hands that lost their insurance.
    pub won:        u64,

    /// Total number of hands not won and not pushed, which includes surrendered
    /// hands and lost insured hands that won their insurance.
    pub lost:       u64,

    /// Total number of pushed hands, which includes an insured natural against
    /// a natural.
    pub push:       u64,

    /// Total number of player hands that busted.
    pub busted:     u64,

    /// Total number of natural the player received.
    pub blackjack:  u64,

    /// The total number of hands the player doubled down.
    pub doubled:    u64,

    /// The total number of times the player split a hand, this is not the total
    /// number of hands got from splitting.
    pub split:      u64,

    /// The number of hands the player insured.
    pub insured:    u64,

    /// The number of hands the player surrendered.
    pub surrender:  u64,
}

impl Default for HandStats {
    /// Create a default [`HandStats`] with all values set to zero.
    fn default() -> Self {
        HandStats {
            total: 0,
            won: 0,
            lost: 0,
            push: 0,
            busted: 0,
            blackjack: 0,
            doubled: 0,
            split: 0,
            insured: 0,
            surrender: 0,
        }
    }
}

impl From<(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64)> for HandStats {
    /// Convenience function for creating a [`HandStats`] from a tuple with:
    ///   * total,
    ///   * won,
    ///   * lost,
    ///   * push,
    ///   * busted,
    ///   * blackjack,
    ///   * doubled,
    ///   * split,
    ///   * insured,
    ///   * surrender,
    fn from(v: (u64, u64, u64, u64, u64, u64, u64, u64, u64, u64)) -> Self {
        HandStats {
            total: v.0,
            won: v.1,
            lost: v.2,
            push: v.3,
            busted: v.4,
            blackjack: v.5,
            doubled: v.6,
            split: v.7,
            insured: v.8,
            surrender: v.9,
        }
    }
}

impl HandStats {
    /// Update the current stats based a player hand and its game outcome.
    ///
    /// `total` is always incremented; the outcome will increment `won`, `push`,
    /// or `lost`; other members are independently updated from the properties
    /// of the given hand.
    pub fn update(&mut self, hand: &Hand, outcome: HandOutcome) {
        self.total += 1;

        match outcome {
            HandOutcome::Win => self.won += 1,
            HandOutcome::Push => self.push += 1,
            HandOutcome::Lose => self.lost += 1,
        }

        if hand.is_busted() {
            self.busted += 1;
        }
        if hand.is_bj() {
            self.blackjack += 1;
        }
        if hand.is_doubled() {
            self.doubled += 1;
        }
        if hand.is_split() {
            self.split += 1;
        }
        if hand.is_insured() {
            self.insured += 1;
        }
        if hand.is_surrendered() {
            self.surrender += 1;
        }
    }
}

impl Add for HandStats {
    type Output = Self;

    /// Sum two hand stats, all fields are summed together.
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            total: self.total + rhs.total,
            won: self.won + rhs.won,
            lost: self.lost + rhs.lost,
            push: self.push + rhs.push,
            busted: self.busted + rhs.busted,
            blackjack: self.blackjack + rhs.blackjack,
            doubled: self.doubled + rhs.doubled,
            split: self.split + rhs.split,
            insured: self.insured + rhs.insured,
            surrender: self.surrender + rhs.surrender,
        }
    }
}

impl AddAssign for HandStats {
    /// Sum two hand stats, all fields are summed together.
    fn add_assign(&mut self, rhs: Self) {
        self.total += rhs.total;
        self.won += rhs.won;
        self.lost += rhs.lost;
        self.push += rhs.push;
        self.busted += rhs.busted;
        self.blackjack += rhs.blackjack;
        self.doubled += rhs.doubled;
        self.split += rhs.split;
        self.insured += rhs.insured;
        self.surrender += rhs.surrender;
    }
}

#[cfg(test)]
mod tests {
    use crate::hand_stats::HandStats;
    use crate::hand::Hand;
    use crate::hand_logic::hand_result;

    #[test]
    fn it_update_hand_stats() {
        //               Tt Wo Lo Pu Bu BJ Db Sp In Su
        test_hand_stats(&[10, 8], &[10, 7], 0,
                        (1, 1, 0, 0, 0, 0, 0, 0, 0, 0));
        test_hand_stats(&[10, 8], &[1, 7], INSURED,
                        (1, 0, 0, 1, 0, 0, 0, 0, 1, 0));
    }

    const DOUBLED: u32      = 1 << 0;
    const SURRENDERED: u32  = 1 << 1;
    const INSURED: u32      = 1 << 2;
    const SPLIT: u32        = 1 << 3;

    fn test_hand_stats(player: &[u8],
                       dealer: &[u8],
                       opts: u32,
                       values: (u64, u64, u64, u64, u64, u64, u64, u64, u64, u64)) {
        let mut stats = HandStats::default();

        let mut player = Hand::from(player);
        if opts & DOUBLED > 0 { player.double_down(); }
        if opts & SURRENDERED > 0 { player.surrender(); }
        if opts & INSURED > 0 { player.insure(); }
        if opts & SPLIT > 0 { player.split(); }

        let dealer = Hand::from(dealer);

        let (outcome, _) = hand_result(&player, &dealer);
        stats.update(&player, outcome);

        assert_eq!(stats, HandStats::from(values));
    }
}
