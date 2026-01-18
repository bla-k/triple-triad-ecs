use std::ops::Not;

// TODO move this next to carddb
pub type CardId = usize;

// =========================================== Entity ==============================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entity(pub(crate) usize);

impl Entity {
    pub fn id(&self) -> usize {
        self.0
    }
}

// ======================================== Match State ============================================

#[derive(Clone, Copy, Debug)]
pub enum MatchState {
    GameStart,
    Turn { phase: TurnPhase, player: Player },
    GameOver,
}

#[derive(Clone, Copy, Debug)]
pub enum TurnPhase {
    Start,
    SelectCard {
        cursor: usize,
        entity: Option<Entity>,
    },
    PlaceCard {
        cursor: (usize, usize),
        entity: Entity,
    },
    ResolveRules {
        entity: Entity,
    },
    End,
}

// ================================== Game =====================================

pub struct Game {
    pub components: Components,
    pub state: MatchState,
}

impl Game {
    pub fn init() -> Self {
        Game {
            components: Components::default(),
            state: MatchState::GameStart,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Player {
    P1 = 0,
    P2 = 1,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    /// (x, y) (0..3, 0..3)
    Board(usize, usize),
    /// index 0..5
    Hand(usize),
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
