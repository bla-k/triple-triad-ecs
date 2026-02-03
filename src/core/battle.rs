use std::{iter::FusedIterator, ops::Not};

use crate::data::CardId;

pub const BOARD_SIZE: usize = 3;

// =========================================== Entity ==============================================

/// Represents the unique identifier for an ECS Entity in a match.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entity(u8);

impl Entity {
    pub const MAX: u8 = 10;

    pub fn new(index: u8) -> Option<Self> {
        if index < Self::MAX {
            Some(Self(index))
        } else {
            None
        }
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn iter() -> EntityIter {
        EntityIter(0..Self::MAX)
    }
}

/// Provides a type safe way to iterate over valid entities.
pub struct EntityIter(std::ops::Range<u8>);

impl Iterator for EntityIter {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Entity)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for EntityIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Entity)
    }
}

impl ExactSizeIterator for EntityIter {}

impl FusedIterator for EntityIter {}

// =========================================== Player ==============================================

/// Player identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

// ========================================== Position =============================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Position {
    Board(BoardCoords),
    Hand(usize),
}

/// Board coordinates with guaranteed validity.
///
/// ```txt
///      x=0 x=1 x=2
///     +---+---+---+
/// y=0 |   |   |   |
///     +---+---+---+
/// y=1 |   |   |   |
///     +---+---+---+
/// y=2 |   |   |   |
///     +---+---+---+
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoardCoords(usize, usize);

impl BoardCoords {
    pub fn new(x: usize, y: usize) -> Option<Self> {
        (x < BOARD_SIZE && y < BOARD_SIZE).then_some(Self(x, y))
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.1 * BOARD_SIZE + self.0
    }

    #[inline]
    pub fn x(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn y(&self) -> usize {
        self.1
    }

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Down if self.1 < BOARD_SIZE - 1 => Some(Self(self.0, self.1 + 1)),

            Direction::Left if self.0 > 0 => Some(Self(self.0 - 1, self.1)),

            Direction::Right if self.0 < BOARD_SIZE - 1 => Some(Self(self.0 + 1, self.1)),

            Direction::Up if self.1 > 0 => Some(Self(self.0, self.1 - 1)),

            _ => None,
        }
    }
}

/// Cardinal direction on the board.
#[derive(Debug)]
pub enum Direction {
    Down,
    Left,
    Right,
    Up,
}

// ========================================= Components ============================================

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
