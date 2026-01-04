use std::ops::Not;

pub type CardId = usize;
pub type Entity = usize;

pub const GRID_SIZE: usize = 3;

#[derive(Debug)]
pub enum Event {
    // Director Events
    Quit,
    CardDeselected,
    CardSelected,
    CardPlaced,
    // Selection Events
    SelectCard,
    SelectCursorDown,
    SelectCursorUp,
    // Placement Events
    PlaceCard,
    PlaceCursorDown,
    PlaceCursorLeft,
    PlaceCursorRight,
    PlaceCursorUp,
    // Rule Events
    RuleFlip(Entity),
    // Win Condition Events
    PlayerWins(Player),
    DrawGame,
}

#[derive(Debug)]
pub enum Phase {
    GameStart,      // randomly determine currently active player
    TurnStart,      // FIXME
    SelectCard,     // player chooses card from hand using cursor
    PlaceCard,      // player chooses destination board cell using cursor
    CheckNeighbors, // placed card is compared with neighbor cards, additional rules are evaluated if active
    TurnEnd,        // check winning condition
    SwitchPlayer,   // flip active player
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    P1,
    P2,
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

#[derive(Debug, PartialEq)]
pub enum Position {
    /// (x, y) (0..3, 0..3)
    Board(usize, usize),
    /// index 0..5
    Hand(usize),
}

pub struct Game {
    /// eg. Currently selected card, placed card, attacker card, etc.
    pub active_entity: Option<Entity>,
    pub cursor: Option<Position>,
    pub phase: Phase,
    pub turn: Option<Player>,
    pub components: Components,
}

impl Game {
    pub fn init() -> Self {
        Game {
            // resources
            active_entity: None,
            cursor: None,
            phase: Phase::GameStart,
            turn: None,
            components: Components::default(),
        }
    }
}

// =============================== Components ==================================

pub struct Components {
    pub card: Vec<Option<CardId>>,
    pub owner: Vec<Option<Player>>,
    pub position: Vec<Option<Position>>,
}

impl Default for Components {
    fn default() -> Self {
        let card = vec![
            Some(1),
            Some(4),
            Some(8),
            Some(12),
            Some(16),
            Some(5),
            Some(10),
            Some(15),
            Some(20),
            Some(25),
        ];
        let owner = vec![
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
        let position = vec![
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
            card,
            owner,
            position,
        }
    }
}

//use std::ops::Not;
//
//use crate::data::{CardDb, CardStats};
//
///// Instance identifier.
//pub type Entity = usize;
//
///// Card identifier in card library.
//pub type CardId = usize;
//
//const GRID_SIZE: u8 = 3;
//
///// Convert (x, y) board cell coordinates to vector board index.
//pub fn c2i(x: u8, y: u8) -> u8 {
//    y * GRID_SIZE + x
//}
//
//#[derive(Debug, Default)]
//pub struct Game {
//    pub resources: Resources,
//    pub world: World,
//}
//
////#[derive(Debug, Default)]
////pub struct BoardCtx {
////    // pub board: Board,
////    // pub hand: Hand,
////    pub turn: Option<Player>,
////}
//
//// TODO
//// - remove BoardCtx at all
//// - consume turn resource instead
//// - turn methods into utility functions (queries) that combine resources and components.
////impl BoardCtx {
////pub fn hand_size(&self) -> u8 {
////    (0usize..10).filter_map(|entity| )
//
////    match self.turn {
////        Some(Player::P1) => self.hand.p1.len() as u8,
////        Some(Player::P2) => self.hand.p2.len() as u8,
////        None => 0,
////    }
////}
//
////pub fn place_card(&mut self, hand_index: u8, board_coords: (u8, u8)) -> Entity {
////    let entity = match self.turn {
////        Some(Player::P1) => self.hand.p1.remove(hand_index as usize),
////        Some(Player::P2) => self.hand.p2.remove(hand_index as usize),
////        None => unreachable!("Cannot place card out of player's turn."),
////    };
////    let cell_index = c2i(board_coords.0, board_coords.1) as usize;
////    self.board.state[cell_index as usize] = Some(entity);
//
////    entity
////}
////}
//
//#[derive(Debug, Default)]
//pub struct InteractionCtx {
//    pub cursor: Option<Cursor>,
//    pub phase: GamePhase,
//    /// Remembers last cursor position (Hand) when choosing target board cell.
//    pub selection: Option<Cursor>,
//}
//
//impl InteractionCtx {
//    pub fn move_hand_cursor_down(&mut self, hand_size: u8) {
//        if let Some(Cursor::Hand(j)) = &mut self.cursor {
//            *j = (*j + 1) % hand_size;
//        }
//    }
//
//    pub fn move_hand_cursor_up(&mut self, hand_size: u8) {
//        if let Some(Cursor::Hand(j)) = &mut self.cursor {
//            *j = ((*j as i8) - 1).rem_euclid(hand_size as i8) as u8;
//        }
//    }
//
//    pub fn undo_select_card(&mut self) {
//        self.cursor = self.selection.take()
//    }
//
//    pub fn move_board_cursor_down(&mut self) {
//        if let Some(Cursor::Board(_, y)) = &mut self.cursor {
//            *y = (*y + 1) % GRID_SIZE;
//        }
//    }
//
//    pub fn move_board_cursor_left(&mut self) {
//        if let Some(Cursor::Board(x, _)) = &mut self.cursor {
//            *x = (*x as i8 - 1).rem_euclid(GRID_SIZE as i8) as u8;
//        }
//    }
//
//    pub fn move_board_cursor_right(&mut self) {
//        if let Some(Cursor::Board(x, _)) = &mut self.cursor {
//            *x = (*x + 1) % GRID_SIZE;
//        }
//    }
//
//    pub fn move_board_cursor_up(&mut self) {
//        if let Some(Cursor::Board(_, y)) = &mut self.cursor {
//            *y = (*y as i8 - 1).rem_euclid(GRID_SIZE as i8) as u8
//        }
//    }
//
//    // FIXME this will change when selection is moved to bitset
//    pub fn select_card(&mut self) -> Option<Cursor> {
//        let Some(Cursor::Hand(j)) = self.cursor else {
//            return None;
//        };
//
//        self.selection = self.cursor.take();
//        Some(Cursor::Hand(j))
//    }
//
//    pub fn placed_card<'a>(
//        &self,
//        entity: Entity,
//        components: &'a Components,
//        card_db: &'a CardDb,
//    ) -> Option<(u8, u8, CardView<'a>)> {
//        let Some(Cursor::Board(x, y)) = self.cursor else {
//            return None;
//        };
//        //let entity = components.position.iter().find(predicate)
//
//        Some((x, y, card_at(entity, components, card_db)?))
//    }
//}
//
//#[derive(Debug)]
//pub struct CardView<'a> {
//    pub entity: usize,
//    pub owner: &'a Player,
//    pub position: &'a Cursor,
//    pub stats: &'a CardStats,
//}
//
//pub fn card_at<'a>(
//    entity: Entity,
//    components: &'a Components,
//    card_db: &'a CardDb,
//) -> Option<CardView<'a>> {
//    let owner = components.owner[entity].as_ref()?;
//    let card_id = components.card[entity]?;
//    let position = components.position[entity].as_ref()?;
//    let stats = &card_db.stats[card_id];
//
//    Some(CardView {
//        entity,
//        owner,
//        position,
//        stats,
//    })
//}
//
//#[derive(Debug, Default)]
//pub struct Resources {
//    //  pub board: BoardCtx,
//    pub interaction: InteractionCtx,
//    pub turn: Option<Player>,
//}
//
//#[derive(Debug)]
//pub struct Board {
//    pub state: Vec<Option<Entity>>,
//}
//
//impl Board {
//    pub const GRID_SIZE: u8 = 3;
//
//    pub fn into_index(x: u8, y: u8) -> usize {
//        (y * Self::GRID_SIZE + x) as usize
//    }
//}
//
//impl Default for Board {
//    fn default() -> Self {
//        let mut state = Vec::with_capacity(9);
//        state.resize(9, None);
//        Board { state }
//    }
//}
//
//// TODO rename into something more generic that can be used for selection, cursor and position.
//#[derive(Debug, PartialEq)]
//pub enum Cursor {
//    Hand(u8),
//    Board(u8, u8),
//}
//
//#[derive(Clone, Copy, Debug, Default)]
//#[repr(u8)]
//pub enum GamePhase {
//    #[default]
//    GameStart, // randomly determine currently active player
//    TurnStart,      // FIXME
//    SelectCard,     // player chooses card from hand using cursor
//    PlaceCard,      // player chooses destination board cell using cursor
//    CheckNeighbors, // placed card is compared with neighbor cards, additional rules are evaluated if active
//    TurnEnd,        // check winning condition
//    SwitchPlayer,   // flip active player
//    GameOver,
//}
//
//#[derive(Debug)]
//pub struct Hand {
//    pub p1: Vec<Entity>,
//    pub p2: Vec<Entity>,
//}
//
//// FIXME this should be overridden by gameplay
//impl Default for Hand {
//    fn default() -> Self {
//        let p1 = vec![0, 1, 2, 3, 4];
//        let p2 = vec![5, 6, 7, 8, 9];
//
//        Hand { p1, p2 }
//    }
//}
//
//#[derive(Clone, Copy, Debug, PartialEq)]
//#[repr(u8)]
//pub enum Player {
//    P1 = 0,
//    P2 = 1,
//}
//
//impl Not for Player {
//    type Output = Self;
//
//    fn not(self) -> Self::Output {
//        match self {
//            Player::P1 => Player::P2,
//            Player::P2 => Player::P1,
//        }
//    }
//}
//
//#[derive(Debug, Default)]
//pub struct World {
//    pub components: Components,
//}
//
//#[derive(Debug)]
//pub struct Components {
//    pub card: Vec<Option<CardId>>,
//    pub owner: Vec<Option<Player>>,
//    pub position: Vec<Option<Cursor>>,
//}
//
//impl Default for Components {
//    fn default() -> Self {
//        Components {
//            card: vec![
//                Some(0),
//                Some(2),
//                Some(4),
//                Some(6),
//                Some(8),
//                Some(1),
//                Some(3),
//                Some(5),
//                Some(7),
//                Some(9),
//            ],
//            owner: vec![
//                Some(Player::P1),
//                Some(Player::P1),
//                Some(Player::P1),
//                Some(Player::P1),
//                Some(Player::P1),
//                Some(Player::P2),
//                Some(Player::P2),
//                Some(Player::P2),
//                Some(Player::P2),
//                Some(Player::P2),
//            ],
//            position: vec![
//                Some(Cursor::Hand(0)),
//                Some(Cursor::Hand(1)),
//                Some(Cursor::Hand(2)),
//                Some(Cursor::Hand(3)),
//                Some(Cursor::Hand(4)),
//                Some(Cursor::Hand(0)),
//                Some(Cursor::Hand(1)),
//                Some(Cursor::Hand(2)),
//                Some(Cursor::Hand(3)),
//                Some(Cursor::Hand(4)),
//            ],
//        }
//    }
//}
