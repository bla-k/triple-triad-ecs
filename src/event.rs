use std::collections::VecDeque;

use crate::game::{Entity, Player};

#[derive(Debug)]
pub enum Command {
    Cancel,
    Confirm,
    MoveCursor(Direction),
    Quit,
}

#[derive(Debug)]
pub enum Direction {
    Down,
    Left,
    Right,
    Up,
}

#[derive(Debug)]
pub enum GameEvent {
    CardSelected { target: Entity },
    CardDeselected,
    CardPlaced,
    CaptureDetected { target: Entity },
    CardFlipped,
    MatchEnded(MatchResult),
}

#[derive(Debug)]
pub enum MatchResult {
    Draw,
    Winner(Player),
}

#[derive(Debug, Default)]
pub struct Bus {
    pub commands: VecDeque<Command>,
    pub events: VecDeque<GameEvent>,
    pub flips: VecDeque<Entity>,
}

