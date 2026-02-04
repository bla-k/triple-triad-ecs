use std::{
    iter::FusedIterator,
    ops::{Index, IndexMut, Not},
    slice::Iter,
};

use crate::data::CardId;

pub const BOARD_SIZE: usize = 3;

// =========================================== Battle ==============================================

pub struct Battle {
    pub components: Components,
    pub state: State,
}

impl Battle {
    pub fn init() -> Self {
        Self {
            components: Components::default(),
            state: State::default(),
        }
    }
}

// ========================================= Components ============================================

pub struct Components {
    pub card: ComponentArray<CardId>,
    pub owner: ComponentArray<Player>,
    pub position: ComponentArray<Position>,
}

impl Default for Components {
    fn default() -> Self {
        let card = [
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
        let owner = [
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
        let position = [
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
            card: ComponentArray(card),
            owner: ComponentArray(owner),
            position: ComponentArray(position),
        }
    }
}

pub struct ComponentArray<T>([Option<T>; Entity::MAX as usize]);

impl<T> ComponentArray<T> {
    pub fn iter(&self) -> Iter<'_, Option<T>> {
        self.0.iter()
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.0[entity.index()].as_ref()
    }

    pub fn insert(&mut self, entity: Entity, value: T) -> Option<T> {
        self.0[entity.index()].replace(value)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.0[entity.index()].take()
    }
}

impl<T> Index<Entity> for ComponentArray<T> {
    type Output = Option<T>;

    fn index(&self, entity: Entity) -> &Self::Output {
        &self.0[entity.index()]
    }
}

impl<T> IndexMut<Entity> for ComponentArray<T> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        &mut self.0[entity.index()]
    }
}

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
    pub const CENTER: Self = Self(1, 1);

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

    pub fn moved_down(&self) -> Self {
        Self(self.0, (self.1 + 1) % BOARD_SIZE)
    }

    pub fn moved_left(&self) -> Self {
        Self((self.0 + BOARD_SIZE - 1) % BOARD_SIZE, self.1)
    }

    pub fn moved_right(&self) -> Self {
        Self((self.0 + 1) % BOARD_SIZE, self.1)
    }

    pub fn moved_up(&self) -> Self {
        Self(self.0, (self.1 + BOARD_SIZE - 1) % BOARD_SIZE)
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

//============================================ State ===============================================

#[derive(Clone, Copy, Debug, Default)]
pub enum State {
    #[default]
    Start,
    Turn {
        phase: TurnPhase,
        player: Player,
    },
    End {
        result: BattleResult,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum TurnPhase {
    Start,
    SelectCard { cursor: usize, entity: Entity },
    PlaceCard { cursor: BoardCoords, entity: Entity },
    ResolveRules { entity: Entity },
    End,
}

#[derive(Clone, Copy, Debug)]
pub enum BattleResult {
    Draw,
    Win(Player),
}
