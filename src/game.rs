use crate::{
    core::battle::{Entity, Player, Position},
    data::CardId,
};

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
