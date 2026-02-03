use crate::{
    core::battle::{Components, Entity, Player, Position},
    data::CardDb,
    query::{CardView, get_card_view},
    sdl::AssetManager,
    ui::{Layout, Theme, UI},
};
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

// ============================= Render Context ================================

pub struct RenderCtx<'a, 'b> {
    pub asset_manager: &'a mut AssetManager<'b>,
    pub canvas: &'a mut Canvas<Window>,
    pub ui: &'a UI,
}

// ============================ Render Functions ===============================

pub fn render_board(ctx: &mut RenderCtx) -> Result<(), String> {
    let (sprite, texture) = ctx
        .asset_manager
        .get_sprexture("cell")
        .ok_or("ERR: Missing asset")?;

    let Theme { fg, .. } = ctx.ui.palette.mono;
    texture.set_color_mod(fg.r, fg.g, fg.b);

    for rect in ctx.ui.layout.board {
        ctx.canvas.copy(texture, sprite.region, rect)?;
    }

    texture.set_color_mod(255, 255, 255);

    Ok(())
}

// vvv TODO vvv

#[rustfmt::skip]
pub fn render_card(
    ctx: &mut RenderCtx,
    entity: Entity,
    active_entity: Option<Entity>,
    components: &Components,
    card_db: &CardDb
) -> Result<(), String> {
//  let UI { layout, .. } = ui;

    let Some(card_view) = get_card_view(entity, components, card_db) else {
        return Err(format!("No such entity: '{entity:?}'"));
    };

    let dst = get_dest_rect(active_entity, &card_view, &ctx.ui.layout);
    let (texture, src) = get_texture(&card_view, ctx.asset_manager)?;

    ctx.canvas.copy(texture, src, dst)?;

    //
    // >>> TODO render stats <<<
    //
    let stat_parts = [
        (card_view.stats.top, ctx.ui.layout.card.stats.top),
        (card_view.stats.lft, ctx.ui.layout.card.stats.lft),
        (card_view.stats.rgt, ctx.ui.layout.card.stats.rgt),
        (card_view.stats.btm, ctx.ui.layout.card.stats.btm),
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
        render_char(value, char_mode, dst, ctx)?;
    }

    Ok(())
}

/// Returns card's destination region, extracting it from `Layout`.
fn get_dest_rect(active_entity: Option<Entity>, card_view: &CardView, layout: &Layout) -> Rect {
    let Layout { board, hand, .. } = layout;
    let &CardView {
        entity,
        position,
        owner,
        ..
    } = card_view;

    match (active_entity, position, owner) {
        (Some(hovered), &Position::Hand(j), Player::P1) if hovered == entity => {
            hand.p1[j].right_shifted(Layout::HOVER_SHIFT)
        }

        (_, &Position::Hand(j), Player::P1) => hand.p1[j],

        (Some(hovered), &Position::Hand(j), Player::P2) if hovered == entity => {
            hand.p2[j].left_shifted(Layout::HOVER_SHIFT)
        }

        (_, &Position::Hand(j), Player::P2) => hand.p2[j],

        (_, &Position::Board(board_coords), _) => board[board_coords.index()],
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

pub fn render_char(c: char, mode: CharMode, dst: Rect, ctx: &mut RenderCtx) -> Result<(), String> {
    let Some((font, texture)) = ctx.asset_manager.get_font() else {
        eprintln!("ERR: font not loaded");
        return Ok(());
    };
    let Some(glyph) = font.glyphs[c as usize] else {
        eprintln!("ERR: no such glyph");
        return Ok(());
    };

    let Theme { bg, fg } = ctx.ui.palette.mono;

    match mode {
        CharMode::RegularLight => {
            texture.set_color_mod(fg.r, fg.g, fg.b);
            ctx.canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
        CharMode::RegularDark => {
            texture.set_color_mod(bg.r, bg.g, bg.b);
            ctx.canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
        CharMode::BoldLight => {
            texture.set_color_mod(bg.r, bg.g, bg.b);
            ctx.canvas.copy(texture, glyph.1, dst)?;
            texture.set_color_mod(fg.r, fg.g, fg.b);
            ctx.canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
        CharMode::BoldDark => {
            texture.set_color_mod(fg.r, fg.g, fg.b);
            ctx.canvas.copy(texture, glyph.1, dst)?;
            texture.set_color_mod(bg.r, bg.g, bg.b);
            ctx.canvas.copy(texture, glyph.0, dst)?;
            texture.set_color_mod(255, 255, 255);
        }
    }

    Ok(())
}
