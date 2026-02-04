use std::{
    iter::FusedIterator,
    ops::{Index, IndexMut, Not},
    slice::Iter,
};

use crate::core::data::CardId;

pub const BOARD_SIZE: usize = 3;

pub const HAND_SIZE: usize = 5;

pub const P1_ENTITIES: EntityIter = EntityIter(0..5);

pub const P2_ENTITIES: EntityIter = EntityIter(5..10);

// =========================================== Battle ==============================================

pub struct Battle {
    pub components: Components,
    pub state: State,
}

impl From<BattleSetup> for Battle {
    fn from(value: BattleSetup) -> Self {
        let mut components = Components::default();

        let Components {
            card,
            owner,
            position,
        } = &mut components;

        for entity in P1_ENTITIES {
            card.insert(entity, value.p1_hand[entity.index()]);
            owner.insert(entity, Player::P1);
            position.insert(entity, Position::Hand(entity.index()));
        }

        for entity in P2_ENTITIES {
            card.insert(entity, value.p2_hand[entity.index() - HAND_SIZE]);
            owner.insert(entity, Player::P2);
            position.insert(entity, Position::Hand(entity.index() - HAND_SIZE));
        }

        Self {
            components,
            state: State::default(),
        }
    }
}

#[derive(Debug)]
pub struct BattleSetup {
    pub p1_hand: [CardId; HAND_SIZE],
    pub p2_hand: [CardId; HAND_SIZE],
}

// ========================================= Components ============================================

#[derive(Debug, Default)]
pub struct Components {
    pub card: ComponentArray<CardId>,
    pub owner: ComponentArray<Player>,
    pub position: ComponentArray<Position>,
}

#[derive(Debug)]
pub struct ComponentArray<T>([Option<T>; Entity::MAX as usize]);

impl<T> Default for ComponentArray<T>
where
    T: Copy,
{
    fn default() -> Self {
        Self([None; Entity::MAX as usize])
    }
}

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
