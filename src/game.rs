use std::ops::Not;

pub type CardId = usize;
pub type Entity = usize;

pub const GRID_SIZE: usize = 3;

#[derive(Debug)]
pub enum Event {
    // Director Events
    Quit,
    CardDeselected,
    CardSelected,
    CardPlaced,
    // Selection Events
    SelectCard,
    SelectCursorDown,
    SelectCursorUp,
    // Placement Events
    PlaceCard,
    PlaceCursorDown,
    PlaceCursorLeft,
    PlaceCursorRight,
    PlaceCursorUp,
    // Rule Events
    RuleFlip(Entity),
    // Win Condition Events
    PlayerWins(Player),
    DrawGame,
}

#[derive(Debug)]
pub enum Phase {
    GameStart,      // randomly determine currently active player
    TurnStart,      // FIXME
    SelectCard,     // player chooses card from hand using cursor
    PlaceCard,      // player chooses destination board cell using cursor
    CheckNeighbors, // placed card is compared with neighbor cards, additional rules are evaluated if active
    TurnEnd,        // check winning condition
    SwitchPlayer,   // flip active player
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    P1,
    P2,
}

impl Not for Player {
    type Output = Player;

    fn not(self) -> Self::Output {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Position {
    /// (x, y) (0..3, 0..3)
    Board(usize, usize),
    /// index 0..5
    Hand(usize),
}

pub struct Game {
    /// eg. Currently selected card, placed card, attacker card, etc.
    pub active_entity: Option<Entity>,
    pub cursor: Option<Position>,
    pub phase: Phase,
    pub turn: Option<Player>,
    pub components: Components,
}

impl Game {
    pub fn init() -> Self {
        Game {
            // resources
            active_entity: None,
            cursor: None,
            phase: Phase::GameStart,
            turn: None,
            components: Components::default(),
        }
    }
}

// =============================== Components ==================================

pub struct Components {
    pub card: Vec<Option<CardId>>,
    pub owner: Vec<Option<Player>>,
    pub position: Vec<Option<Position>>,
}

impl Default for Components {
    fn default() -> Self {
        let card = vec![
            Some(1),
            Some(4),
            Some(8),
            Some(12),
            Some(16),
            Some(5),
            Some(10),
            Some(15),
            Some(20),
            Some(109),
        ];
        let owner = vec![
            Some(Player::P1),
            Some(Player::P1),
            Some(Player::P1),
            Some(Player::P1),
            Some(Player::P1),
            Some(Player::P2),
            Some(Player::P2),
            Some(Player::P2),
            Some(Player::P2),
            Some(Player::P2),
        ];
        let position = vec![
            Some(Position::Hand(0)),
            Some(Position::Hand(1)),
            Some(Position::Hand(2)),
            Some(Position::Hand(3)),
            Some(Position::Hand(4)),
            Some(Position::Hand(0)),
            Some(Position::Hand(1)),
            Some(Position::Hand(2)),
            Some(Position::Hand(3)),
            Some(Position::Hand(4)),
        ];
        Components {
            card,
            owner,
            position,
        }
    }
}

