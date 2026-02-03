use crate::{
    core::battle::{Entity, Player},
    data::{CardDb, Stats},
    game::{Components, Position},
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
    let owner = components.owner[entity.index()].as_ref()?;
    let position = components.position[entity.index()].as_ref()?;
    let card_id = components.card[entity.index()]?;

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
    owners: &[Option<Player>],
    positions: &[Option<Position>],
) -> Option<Entity> {
    Entity::iter()
        .find(|e| owners[e.index()] == Some(player) && positions[e.index()] == Some(position))
}

/// Returns the entity corresponding to the query `Position`.
///
/// This is useful when you have to match a card that is placed on the board, but you don't care
/// about card's ownership.
pub fn get_placed_entity(position: Position, positions: &[Option<Position>]) -> Option<Entity> {
    Entity::iter().find(|e| positions[e.index()] == Some(position))
}

/// Returns current player's hand size.
pub fn hand_size(
    player: Player,
    owners: &[Option<Player>],
    positions: &[Option<Position>],
) -> usize {
    owners
        .iter()
        .zip(positions.iter())
        .filter(|&(&owner, &position)| {
            owner == Some(player) && matches!(position, Some(Position::Hand(_)))
        })
        .count()
}
