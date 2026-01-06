use sdl2::{pixels::Color, rect::Rect};

#[derive(Default)]
pub struct UI {
    pub layout: Layout,
    pub palette: Palette,
}

// ================================ Palette ====================================

#[derive(Default)]
pub struct Palette {
    pub wireframe: Wireframe,
    pub mono: Theme,
}

pub struct Theme {
    pub bg: Color,
    pub fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            bg: Color::RGB(5, 15, 26),
            fg: Color::RGB(163, 184, 204),
        }
    }
}

pub struct Wireframe {
    pub background: Color,
    pub board: Color,
    pub p1: Color,
    pub p2: Color,
}

impl Default for Wireframe {
    fn default() -> Self {
        Wireframe {
            background: Color::RGB(0, 0, 0),
            board: Color::RGB(255, 0, 255),
            p1: Color::RGB(0, 255, 255),
            p2: Color::RGB(0, 255, 0),
        }
    }
}

// ================================= Layout ====================================

pub struct Layout {
    pub board: [Rect; 9],
    pub card: Card,
    pub hand: Hand,
    pub turn_indicator: TurnIndicator,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            board: [
                Rect::new(194, 94, 128, 128),
                Rect::new(336, 94, 128, 128),
                Rect::new(478, 94, 128, 128),
                Rect::new(194, 236, 128, 128),
                Rect::new(336, 236, 128, 128),
                Rect::new(478, 236, 128, 128),
                Rect::new(194, 378, 128, 128),
                Rect::new(336, 378, 128, 128),
                Rect::new(478, 378, 128, 128),
            ],
            card: Card::default(),
            hand: Hand::default(),
            turn_indicator: TurnIndicator::default(),
        }
    }
}

pub struct Card {
    pub padding: u8,
    pub stats: Stats,
}

impl Default for Card {
    fn default() -> Self {
        Card {
            padding: 4,
            stats: Stats::default(),
        }
    }
}

/// Stats geometry, to be considered relative to current card's Rect.
pub struct Stats {
    pub top: (i32, i32),
    pub rgt: (i32, i32),
    pub btm: (i32, i32),
    pub lft: (i32, i32),
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            top: (28, 12),
            rgt: (44, 21),
            btm: (28, 32),
            lft: (12, 22),
        }
    }
}

pub struct Hand {
    pub p1: [Rect; 5],
    pub p2: [Rect; 5],
}

impl Default for Hand {
    fn default() -> Self {
        Hand {
            p1: [
                Rect::new(26, 95, 126, 126),
                Rect::new(26, 174, 126, 126),
                Rect::new(26, 253, 126, 126),
                Rect::new(26, 332, 126, 126),
                Rect::new(26, 411, 126, 126),
            ],
            p2: [
                Rect::new(648, 95, 126, 126),
                Rect::new(648, 174, 126, 126),
                Rect::new(648, 253, 126, 126),
                Rect::new(648, 332, 126, 126),
                Rect::new(648, 411, 126, 126),
            ],
        }
    }
}

pub struct TurnIndicator {
    pub p1: Rect,
    pub p2: Rect,
}

impl Default for TurnIndicator {
    fn default() -> Self {
        TurnIndicator {
            p1: Rect::new(66, 37, 65, 34),
            p2: Rect::new(669, 37, 65, 34),
        }
    }
}
