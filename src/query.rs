use crate::{
    core::battle::{ComponentArray, Components, Entity, Player, Position},
    data::{CardDb, Stats},
};

// ================================ CardView ===================================

pub struct CardView<'a> {
    pub id: usize,
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
        id: card_id,
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
    player: Player,
    position: Position,
    owners: &ComponentArray<Player>,
    positions: &ComponentArray<Position>,
) -> Option<Entity> {
    Entity::iter().find(|&e| owners[e] == Some(player) && positions[e] == Some(position))
}

/// Returns the entity corresponding to the query `Position`.
///
/// This is useful when you have to match a card that is placed on the board, but you don't care
/// about card's ownership.
pub fn get_placed_entity(
    position: Position,
    positions: &ComponentArray<Position>,
) -> Option<Entity> {
    Entity::iter().find(|&e| positions[e] == Some(position))
}

/// Returns current player's hand size.
pub fn hand_size(
    player: Player,
    owners: &ComponentArray<Player>,
    positions: &ComponentArray<Position>,
) -> usize {
    Entity::iter()
        .filter(|&e| owners[e] == Some(player) && matches!(positions[e], Some(Position::Hand(_))))
        .count()
}
