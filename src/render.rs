use crate::{
    data::CardDb,
    game::{Components, Entity, Player, Position},
    query::{CardView, get_card_view},
    sdl::AssetManager,
    ui::{Theme, UI},
};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

#[rustfmt::skip]
pub fn render_card(
    canvas: &mut Canvas<Window>,
    entity: Entity,
    ui: &UI,
    asset_manager: &mut AssetManager,
    components: &Components,
    card_db: &CardDb
) -> Result<(), String> {
    let Theme { bg, fg } = ui.palette.mono;

    let Some(card_view) = get_card_view(entity, components, card_db) else {
        eprintln!("ERR: No such entity {}", entity);
        return Ok(());
    };

    // get card's destination region from layout
    let dst = match (card_view.owner, card_view.position) {
        (Player::P1, Position::Hand(j)) => ui.layout.hand.p1[*j],
        (Player::P2, Position::Hand(j)) => ui.layout.hand.p2[*j],
        (_, Position::Board(x, y)) => ui.layout.board[*y * 3 + *x],
    };

    struct CardParts {
        color: Color,
        sprite_id_fn: fn(&CardView) -> &'static str,
    }

    let card_parts = [
        CardParts { color: bg, sprite_id_fn: |_| "card-bg" },
        CardParts { color: fg, sprite_id_fn: |_| "card-border" },
        CardParts { color: fg, sprite_id_fn: |card_view| match card_view.owner {
            Player::P1 => "card-body-light",
            Player::P2 => "card-border-dark",
        }},
    ];

    for CardParts { color, sprite_id_fn } in card_parts {
        let sprite_id = sprite_id_fn(&card_view);
        let sprite = asset_manager.get_sprite(sprite_id).unwrap();
        let texture = asset_manager.get_texture_mut(sprite.texture_id).unwrap();
        texture.set_color_mod(color.r, color.g, color.b);
        canvas.copy(texture, sprite.region, dst)?;
        texture.set_color_mod(255, 255, 255);
    }

    let stat_parts = [
        (card_view.stats.top, ui.layout.card.stats.top),
        (card_view.stats.lft, ui.layout.card.stats.lft),
        (card_view.stats.rgt, ui.layout.card.stats.rgt),
        (card_view.stats.btm, ui.layout.card.stats.btm),
    ];

    let char_mode = match card_view.owner {
        Player::P1 => CharMode::BoldLight,
        Player::P2 => CharMode::BoldDark,
    };

    for (value, coords) in stat_parts {
        let value = stat_char(value);
        let (padding_x, padding_y) = coords;
        let dst = Rect::new(
            dst.x + padding_x,
            dst.y + padding_y,
            AssetManager::GLYPH_WIDTH as u32,
            AssetManager::GLYPH_HEIGHT as u32
        );
        render_char(value, char_mode, dst, canvas, ui, asset_manager)?;
    }

    Ok(())
}

fn stat_char(value: u8) -> char {
    match value {
        x @ 0..=9 => (x + 48) as char,
        10 => 'A',
        _ => unreachable!("unexpected value {value}"),
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum CharMode {
    RegularLight,
    RegularDark,
    BoldLight,
    BoldDark,
}

pub fn render_char(
    c: char,
    mode: CharMode,
    dst: Rect,
    canvas: &mut Canvas<Window>,
    ui: &UI,
    asset_manager: &mut AssetManager,
) -> Result<(), String> {
    let Some((font, texture)) = asset_manager.get_font() else {
        eprintln!("ERR: font not loaded");
        return Ok(());
    };
    let Some(glyph) = font.glyphs[c as usize] else {
        eprintln!("ERR: no such glyph");
        return Ok(());
    };

    let Theme { bg, fg } = ui.palette.mono;

    match mode {
        CharMode::RegularLight => {
            texture.set_color_mod(fg.r, fg.g, fg.b);
            canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
        CharMode::RegularDark => {
            texture.set_color_mod(bg.r, bg.g, bg.b);
            canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
        CharMode::BoldLight => {
            texture.set_color_mod(bg.r, bg.g, bg.b);
            canvas.copy(texture, glyph.1, dst)?;
            texture.set_color_mod(fg.r, fg.g, fg.b);
            canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
        CharMode::BoldDark => {
            texture.set_color_mod(fg.r, fg.g, fg.b);
            canvas.copy(texture, glyph.1, dst)?;
            texture.set_color_mod(bg.r, bg.g, bg.b);
            canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
    }

    Ok(())
}
