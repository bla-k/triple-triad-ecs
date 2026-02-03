use crate::core::battle::{Components, Entity, Player};

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
