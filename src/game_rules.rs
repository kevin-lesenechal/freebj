use serde::Serialize;

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize)]
pub enum GameType {
    /// American holecard game
    Ahc,
    /// European no-holecard game
    Enhc,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize)]
pub enum Soft17 {
    /// Hit soft 17 hands
    S17,
    /// Stand on soft 17 hands
    H17,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize)]
pub enum SurrenderPolicy {
    /// Do not allow surrender
    NoSurrender,
    /// Allow early surrender
    EarlySurrender,
    /// Allow late surrender (AHC only)
    LateSurrender,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
pub enum DoublePolicy {
    /// Do not allow doubling-down for any hand
    NoDouble,
    /// Allow doubling-down any hand of any cards
    AnyHand,
    /// Allow doubling-down on any hand of two cards
    AnyTwo,
    /// Allow doubling-down on any hard hand of 9, 10, or 11
    Hard9To11,
    /// Allow doubling-down on any hard hand of 10 or 11
    Hard10To11
}

#[derive(Serialize, Debug)]
pub struct GameRules {
    pub game_type:      GameType,
    pub soft17:         Soft17,
    pub das:            bool,
    pub bj_pays:        f64,
    pub double_down:    DoublePolicy,
    pub surrender:      SurrenderPolicy,
    pub play_ace_pairs: bool,
    pub max_splits:     u32,
    pub decks:          u32,
    pub penetration_cards: u32,
}

impl Default for GameRules {
    fn default() -> Self {
        GameRules {
            game_type:      GameType::Ahc,
            soft17:         Soft17::S17,
            das:            true,
            bj_pays:        1.5,
            double_down:    DoublePolicy::AnyTwo,
            surrender:      SurrenderPolicy::NoSurrender,
            play_ace_pairs: false,
            max_splits:     4,
            decks:          6,
            penetration_cards: 5 * 52,
        }
    }
}
