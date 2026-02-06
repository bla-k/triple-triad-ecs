use crate::core::data::CardId;

// ========================================= Inventory =============================================

#[derive(Clone, Copy, Debug)]
pub struct Inventory([u8; CardId::MAX as usize]);

impl Inventory {
    pub fn new() -> Self {
        Self([0; CardId::MAX as usize])
    }

    pub fn add(&mut self, card_id: CardId, count: u8) {
        let curr = &mut self.0[card_id.index()];
        *curr = curr.saturating_add(count);
    }

    pub fn remove(&mut self, card_id: CardId, count: u8) {
        let curr = &mut self.0[card_id.index()];
        *curr = curr.saturating_sub(count);
    }

    pub fn iter_distinct(&self) -> impl Iterator<Item = CardId> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(card_id, &count)| (count > 0).then_some(CardId::new_const(card_id as u8)))
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
