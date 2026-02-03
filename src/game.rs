use crate::core::battle::{self, Components};

// ================================== Game =====================================

pub struct Game {
    pub components: Components,
    pub state: battle::State,
}

impl Game {
    pub fn init() -> Self {
        Game {
            components: Components::default(),
            state: battle::State::default(),
        }
    }
}
