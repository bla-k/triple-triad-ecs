//use std::collections::VecDeque;
//
//use crate::game;

use crate::{
    data::{CardDb, Stats},
    game::{Components, Entity, Player, Position},
};

// ================================ CardView ===================================

pub struct CardView<'a> {
    pub entity: Entity,
    pub owner: &'a Player,
    pub position: &'a Position,
    pub stats: &'a Stats,
}

// ================================ queries ====================================

pub fn get_card_view<'a>(
    entity: Entity,
    components: &'a Components,
    card_db: &'a CardDb,
) -> Option<CardView<'a>> {
    let owner = components.owner[entity].as_ref()?;
    let position = components.position[entity].as_ref()?;
    let card_id = components.card[entity]?;

    Some(CardView {
        entity,
        owner,
        position,
        stats: &card_db.stats[card_id],
    })
}

/// Returns the entity corresponding to the query `(Player, Position)`.
///
/// This is useful when you have to match a card that is in player's hand, because `Position` alone
/// may collide and return opponent's entity.
pub fn get_owned_entity(
    player: &Player,
    position: &Position,
    owners: &[Option<Player>],
    positions: &[Option<Position>],
) -> Option<Entity> {
    (0..owners.len()).find(|&entity| {
        owners[entity].as_ref() == Some(player) && positions[entity].as_ref() == Some(position)
    })
}

/// Returns the entity corresponding to the query `Position`.
///
/// This is useful when you have to match a card that is placed on the board, but you don't care
/// about card's ownership.
pub fn get_placed_entity(position: &Position, positions: &[Option<Position>]) -> Option<Entity> {
    (0..positions.len()).find(|&entity| positions[entity].as_ref() == Some(position))
}

pub fn hand_size(
    player: &Player,
    owners: &[Option<Player>],
    positions: &[Option<Position>],
) -> usize {
    (0..owners.len())
        .filter(|&entity| {
            owners[entity].as_ref() == Some(player)
                && matches!(positions[entity], Some(Position::Hand(_)))
        })
        .count()
}

