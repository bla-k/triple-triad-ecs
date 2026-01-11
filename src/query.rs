use crate::{
    data::{CardDb, Stats},
    game::{Components, Entity, Player, Position},
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
    let owner = components.owner[entity.id()].as_ref()?;
    let position = components.position[entity.id()].as_ref()?;
    let card_id = components.card[entity.id()]?;

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
    owners
        .iter()
        .zip(positions.iter())
        .enumerate()
        .find_map(|(j, (&owner, &pos))| {
            if owner == Some(player) && pos == Some(position) {
                Some(Entity(j))
            } else {
                None
            }
        })
}

/// Returns the entity corresponding to the query `Position`.
///
/// This is useful when you have to match a card that is placed on the board, but you don't care
/// about card's ownership.
pub fn get_placed_entity(position: Position, positions: &[Option<Position>]) -> Option<Entity> {
    positions.iter().enumerate().find_map(|(j, &pos)| {
        if pos == Some(position) {
            Some(Entity(j))
        } else {
            None
        }
    })
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
