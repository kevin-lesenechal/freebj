use crate::card::Card;
use crate::hand::Hand;
use crate::game_rules::GameRules;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Decision {
    /// Ask the dealer for an additionnal card, this can be repeated
    Hit,
    /// Keep the current hand and end the round
    Stand,
    /// Double the bet and draw one card, ends the round
    Double,
    /// Split the pair in two hands played separately
    Split,
}

/// This structure represents the game context in which a strategy decision must
/// be taken. It exposes the actions available to the player as well as extra
/// known information (e.g. true count, dealer's holecard), and the game rules.
pub struct GameContext<'a> {
    /// The current game rules at the table
    pub rules:      &'a GameRules,

    /// Wether the player may split its pair or not
    pub may_split:  bool,

    /// Wether the player may double down on his hand
    pub may_double: bool,

    /// The current true count of the shoe
    pub true_count: f32,

    /// The dealer's holecard if it is known (see holecarding option)
    pub holecard:   Option<Card>,
}

pub trait Strategy {
    fn player_turn(
        &self,
        game: &GameContext,
        dealer: Card,
        me: &Hand,
    ) -> Decision;

    fn surrender(
        &self,
        _game: &GameContext,
        _dealer: Card,
        _me: &Hand,
        _is_early: bool,
    ) -> bool {
        false
    }

    fn take_insurance(&self, _game: &GameContext, _me: &Hand) -> bool {
        false
    }
}
