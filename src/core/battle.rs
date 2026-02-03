// =========================================== Entity ==============================================

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
}
