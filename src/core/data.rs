#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CardId(u8);

impl CardId {
    pub const MAX: u8 = 110;

    pub fn new(index: u8) -> Option<Self> {
        if index < Self::MAX {
            Some(Self(index))
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// Caller must ensure `index < CardId::MAX`.
    pub unsafe fn new_unchecked(index: u8) -> Self {
        debug_assert!(
            index < Self::MAX,
            "CardId constructed out of bounds: {}",
            index
        );
        Self(index)
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}
