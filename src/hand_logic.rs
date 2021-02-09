use crate::hand::Hand;
use crate::game_rules::DoublePolicy;

/// The game outcome a played hand.
#[derive(PartialEq, Debug)]
pub enum HandOutcome {
    /// The hand won, which includes naturals, busted dealer, and won insured
    /// hands that lost their insurance.
    Win,

    /// The hand was pushed, which includes an insured natural against one.
    Push,

    /// The hand lose, which includes surrendered hands and lost insured hands
    /// that won their insurance.
    Lose,
}

/// Determines the result and outcome of a played hand against a given dealer
///
/// The result is given as a normalized bet of 1.0, a simple win gives +1.0,
/// a simple loss gives -1.0, a won doubled-down hand +2.0, a lost doubled-down
/// hand a -2.0, a natural +1.5, a surrenderred hand gives -0.5. Insurance is
/// also taken into account, adding 1.0 to the result if the dealer received a
/// blackjack, substracting 0.5 if not.
pub fn hand_result(player: &Hand, dealer: &Hand) -> (HandOutcome, f64) {
    let (outcome, mut res) = if player.is_surrendered() {
        (HandOutcome::Lose, -0.5)
    } else if player.is_busted() {
        (HandOutcome::Lose, -1.0)
    } else {
        if player.is_bj() && !dealer.is_bj() {
            (HandOutcome::Win, 1.5)
        } else if dealer.is_busted() {
            (HandOutcome::Win, 1.0)
        } else {
            let player_val = player.value() + (if player.is_bj() {1} else {0});
            let dealer_val = dealer.value() + (if dealer.is_bj() {1} else {0});

            if player_val == dealer_val {
                (HandOutcome::Push, 0.0)
            } else if player_val > dealer_val {
                (HandOutcome::Win, 1.0)
            } else {
                (HandOutcome::Lose, -1.0)
            }
        }
    };

    if player.is_doubled() {
        res *= 2.0;
    }

    if player.is_insured() {
        if dealer.is_bj() {
            res += 1.0;
        } else {
            res -= 0.5;
        }
    }

    (outcome, res)
}

/// Determines whether a player hand can double-down based on the game policy.
///
/// # Parameters
///
///  * `policy` - The game policy in action concerning doubling-down;
///  * `das`    - Whether double-after-split (DAS) is allowed or not;
///  * `hand`   - The player's hand.
pub fn may_double(policy: DoublePolicy, das: bool, hand: &Hand) -> bool {
    if hand.is_split() && !das {
        return false;
    }

    match policy {
        DoublePolicy::AnyHand => true,
        DoublePolicy::AnyTwo => hand.count() == 2,
        DoublePolicy::Hard9To11 =>
            !hand.is_soft() && (9..=11).contains(&hand.value()),
        DoublePolicy::Hard10To11 =>
            !hand.is_soft() && (10..=11).contains(&hand.value()),
        DoublePolicy::NoDouble => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::hand_logic::{hand_result, may_double, HandOutcome};
    use crate::hand_logic::HandOutcome::*;
    use crate::hand::Hand;

    #[test]
    fn it_returns_the_hand_result() {
        test_hand_result(Push,  0.0, &[10, 10],   &[10, 10],      0);
        test_hand_result(Win,   1.0, &[10, 10],   &[10, 9],       0);
        test_hand_result(Lose, -1.0, &[10, 6],    &[10, 9],       0);

        test_hand_result(Push,  0.0, &[10, 10],   &[10, 10],      DOUBLED);
        test_hand_result(Win,   2.0, &[10, 10],   &[10, 9],       DOUBLED);
        test_hand_result(Lose, -2.0, &[10, 6],    &[10, 9],       DOUBLED);

        test_hand_result(Win,   1.5, &[10, 1],    &[7, 7, 7],     0);
        test_hand_result(Push,  0.0, &[10, 1],    &[1, 10],       0);
        test_hand_result(Lose, -1.0, &[7, 7, 7],  &[10, 1],       0);

        test_hand_result(Lose, -1.0, &[7, 7, 8],  &[10, 7],       0);
        test_hand_result(Lose, -1.0, &[7, 7, 8],  &[10, 6, 9],    0);
        test_hand_result(Win,   1.0, &[8, 8],     &[10, 6, 9],    0);

        test_hand_result(Lose, -0.5, &[8, 8],     &[10, 7],       SURRENDERED);
        test_hand_result(Lose, -0.5, &[8, 8],     &[10, 6, 9],    SURRENDERED);

        test_hand_result(Push,  0.0, &[1, 10],    &[7, 7, 7],     SPLIT);

        test_hand_result(Push,  1.0, &[1, 10],    &[1, 10],       INSURED);
        test_hand_result(Lose,  0.0, &[7, 7, 7],  &[1, 10],       INSURED);
        test_hand_result(Win,   0.5, &[7, 7, 7],  &[1, 9],        INSURED);
        test_hand_result(Lose, -1.5, &[10, 6, 7], &[1, 9],        INSURED);
    }

    #[test]
    fn it_determines_whether_it_can_double_down() {
        use crate::game_rules::DoublePolicy::*;

        assert!(may_double(AnyHand,     true,   &Hand::from(&[4, 7][..])));
        assert!(may_double(AnyHand,     false,  &Hand::from(&[4, 7][..])));
        assert!(may_double(AnyHand,     false,  &Hand::from(&[4, 3, 8][..])));
        let mut hand = Hand::from(&[4, 3, 8][..]);
        hand.split();
        assert!(!may_double(AnyHand,    false,  &hand));
        assert!(may_double(AnyHand,     true,   &hand));

        assert!(may_double(AnyTwo,      true,   &Hand::from(&[4, 7][..])));
        assert!(may_double(AnyTwo,      false,  &Hand::from(&[4, 7][..])));
        assert!(may_double(AnyTwo,      false,  &Hand::from(&[1, 7][..])));
        assert!(!may_double(AnyTwo,     false,  &Hand::from(&[4, 3, 8][..])));

        assert!(!may_double(Hard9To11,  true,   &Hand::from(&[5, 3][..])));
        assert!(may_double(Hard9To11,   true,   &Hand::from(&[4, 5][..])));
        assert!(may_double(Hard9To11,   true,   &Hand::from(&[4, 6][..])));
        assert!(may_double(Hard9To11,   true,   &Hand::from(&[4, 7][..])));
        assert!(!may_double(Hard9To11,  true,   &Hand::from(&[4, 8][..])));
        assert!(!may_double(Hard9To11,  true,   &Hand::from(&[1, 2][..])));

        assert!(!may_double(Hard10To11, true,   &Hand::from(&[5, 3][..])));
        assert!(!may_double(Hard10To11, true,   &Hand::from(&[4, 5][..])));
        assert!(may_double(Hard10To11,  true,   &Hand::from(&[4, 6][..])));
        assert!(may_double(Hard10To11,  true,   &Hand::from(&[4, 7][..])));
        assert!(!may_double(Hard10To11, true,   &Hand::from(&[4, 8][..])));
        assert!(!may_double(Hard10To11, true,   &Hand::from(&[1, 2][..])));

        assert!(!may_double(NoDouble,   true,   &Hand::from(&[5, 3][..])));
        assert!(!may_double(NoDouble,   true,   &Hand::from(&[4, 5][..])));
        assert!(!may_double(NoDouble,   true,   &Hand::from(&[4, 6][..])));
        assert!(!may_double(NoDouble,   true,   &Hand::from(&[4, 7][..])));
        assert!(!may_double(NoDouble,   true,   &Hand::from(&[4, 8][..])));
        assert!(!may_double(NoDouble,   true,   &Hand::from(&[1, 2][..])));
    }

    const DOUBLED: u32      = 1 << 0;
    const SURRENDERED: u32  = 1 << 1;
    const INSURED: u32      = 1 << 2;
    const SPLIT: u32        = 1 << 3;

    fn test_hand_result(expected_outcome: HandOutcome,
                        expected_result: f64,
                        player: &[u8],
                        dealer: &[u8],
                        opts: u32) {
        let mut player = Hand::from(player);
        if opts & DOUBLED > 0 { player.double_down(); }
        if opts & SURRENDERED > 0 { player.surrender(); }
        if opts & INSURED > 0 { player.insure(); }
        if opts & SPLIT > 0 { player.split(); }
        let dealer = Hand::from(dealer);

        let (outcome, result) = hand_result(&player, &dealer);

        assert_eq!(outcome, expected_outcome);
        assert_eq!(result, expected_result);
    }
}
