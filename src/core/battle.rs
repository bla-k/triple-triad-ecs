use std::iter::FusedIterator;

// =========================================== Entity ==============================================

/// Represents the unique identifier for an ECS Entity in a match.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
