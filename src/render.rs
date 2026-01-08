use crate::{
    data::CardDb,
    game::{Components, Entity, Player, Position},
    query::{CardView, get_card_view},
    sdl::AssetManager,
    ui::{Layout, Theme, UI},
};
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

#[rustfmt::skip]
pub fn render_card(
    canvas: &mut Canvas<Window>,
    entity: Entity,
    ui: &UI,
    active_entity: Option<Entity>,
    asset_manager: &mut AssetManager,
    components: &Components,
    card_db: &CardDb
) -> Result<(), String> {
    let UI { layout, .. } = ui;

    let Some(card_view) = get_card_view(entity, components, card_db) else {
        return Err(format!("No such entity: '{entity}'"));
    };

    let dst = get_dest_rect(active_entity, &card_view, layout);
    let (texture, src) = get_texture(&card_view, asset_manager)?;

    canvas.copy(texture, src, dst)?;

    //
    // >>> TODO render stats <<<
    //
    let stat_parts = [
        (card_view.stats.top, layout.card.stats.top),
        (card_view.stats.lft, layout.card.stats.lft),
        (card_view.stats.rgt, layout.card.stats.rgt),
        (card_view.stats.btm, layout.card.stats.btm),
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

/// Returns card's destination region, extracting it from `Layout`.
#[rustfmt::skip]
fn get_dest_rect(active_entity: Option<Entity>, card_view: &CardView, layout: &Layout) -> Rect {
    let Layout { board, hand, .. } = layout;
    let &CardView { entity, position, owner, .. } = card_view;

    match (active_entity, position, owner) {
        (Some(hovered), &Position::Hand(j), Player::P1) if hovered == entity => hand.p1[j].right_shifted(Layout::HOVER_SHIFT),
        (            _, &Position::Hand(j), Player::P1)                      => hand.p1[j],
        (Some(hovered), &Position::Hand(j), Player::P2) if hovered == entity => hand.p2[j].left_shifted(Layout::HOVER_SHIFT),
        (            _, &Position::Hand(j), Player::P2)                      => hand.p2[j],
        (            _, &Position::Board(x, y),      _)                      => board[y * Layout::GRID_SIZE + x],
    }
}

/// Returns card's texture and source region, using the registered `Sprite`.
fn get_texture<'a>(
    card_view: &CardView,
    asset_manager: &'a AssetManager,
) -> Result<(&'a Texture<'a>, Rect), String> {
    let sprites = asset_manager
        .card_sprites
        .get(card_view.id)
        .ok_or(format!("No sprite for card id: '{}'", card_view.id))?;

    let sprite = sprites[*card_view.owner as usize];
    let texture = asset_manager
        .get_texture(sprite.texture_id)
        .ok_or(format!("No texture with id: '{}'", sprite.texture_id))?;

    Ok((texture, sprite.region))
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
