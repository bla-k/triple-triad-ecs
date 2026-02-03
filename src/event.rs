use std::collections::VecDeque;

use crate::core::battle::{BattleResult, Direction, Entity};

#[derive(Debug)]
pub enum Command {
    Cancel,
    Confirm,
    MoveCursor(Direction),
    Quit,
}

#[derive(Debug)]
pub enum GameEvent {
    CardSelected { target: Entity },
    CardDeselected,
    CardPlaced,
    CaptureDetected { target: Entity },
    CardFlipped,
    MatchEnded(BattleResult),
}

#[derive(Debug, Default)]
pub struct Bus {
    pub commands: VecDeque<Command>,
    pub events: VecDeque<GameEvent>,
    pub flips: VecDeque<Entity>,
}
