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
    CardSelected,
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
    pub command_queue: VecDeque<Command>,
    pub event_queue: VecDeque<GameEvent>,
    pub flip_queue: VecDeque<Entity>,
}

