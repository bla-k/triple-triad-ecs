// Let's start from the data structures.
//
// In triple triad there is a 3x3 board and each player holds 5 cards in hand.
//
// A card has
// - four stats (top, right, bottom, left) which can be a number between 0 and 10.
// - a name
// - an optional element
//
// Cards belong to a player, but during gameplay ownership can change.
//
// During a game, only card owner and their position on the board can change.
//
// Card list is an asset and should be loaded from a file.
//
// Layout should be configurable and should come from a file too.
//
// Textures are assets too.

// TODO I am now focusing on two refactors
// 1. remove board and hand Resources, in favor of position component.
//      -> rename Cursor into Position
// 2. propagate selected entity via event bus
// 3? change selection into a bitset
//
// TODO after everything is migrated, I want to study further the problem of frame-snapshots using
// quadtree/spatial hash which implies snapshotting the state at frame start and committing changes
// at frame end using command queues. I should study quadtree real world applications and possibly
// try to implement one.

use sdl2::rect::Rect;
use triple_triad::{
    data::CardDb,
    event::{Bus, Command},
    game::{Game, Player},
    render::RenderCtx,
    sdl::{AssetManager, BakeCardCfg, SdlSystems, Sprite},
    systems::{
        director_system, flip_system, input_system, placement_system, render_system, rule_system,
        selection_system, win_system,
    },
    ui::UI,
};

fn main() -> Result<(), String> {
    let card_db = CardDb::load("config/cards.db").map_err(|e| e.to_string())?;
    let ui = UI::default();

    let SdlSystems {
        mut canvas,
        mut event_pump,
        texture_creator,
    } = SdlSystems::init()?;

    let Bus {
        mut command_queue,
        mut event_queue,
        mut flip_queue,
    } = Bus::default();

    let mut asset_manager = AssetManager::default();
    asset_manager.load_font(&texture_creator, "assets/font.png")?;
    let t_cards = asset_manager.load_texture(&texture_creator, "assets/cards.png")?;
    let t_ui = asset_manager.load_texture(&texture_creator, "assets/ui.png")?;
    #[cfg_attr(any(), rustfmt::skip)]
    {
        asset_manager.define_sprite("cell",             t_cards, Rect::new(  0,  0, 128, 128));
        asset_manager.define_sprite("card-bg",          t_cards, Rect::new(128,  0, 128, 128));
        asset_manager.define_sprite("card-border",      t_cards, Rect::new(256,  0, 128, 128));
        asset_manager.define_sprite("card-body-light",  t_cards, Rect::new(384,  0, 128, 128));
        asset_manager.define_sprite("card-border-dark", t_cards, Rect::new(512,  0, 128, 128));
        asset_manager.define_sprite("cursor",           t_ui,    Rect::new(  0,  0,   9,  17));
    }

    #[cfg_attr(any(), rustfmt::skip)]
    for element in &card_db.elements {
        let config = BakeCardCfg { theme: ui.palette.mono, element: *element, };
        let texture_p1 =
            asset_manager.bake_card_texture(&mut canvas, &texture_creator, Player::P1, config)?;
        let texture_p2 =
            asset_manager.bake_card_texture(&mut canvas, &texture_creator, Player::P2, config)?;

        let region = Rect::new(0, 0, AssetManager::CARD_WIDTH, AssetManager::CARD_HEIGHT);

        asset_manager.card_sprites.push([
            Sprite { region, texture_id: texture_p1, },
            Sprite { region, texture_id: texture_p2, },
        ]);
    }

    let mut game = Game::init();

    let mut render_ctx = RenderCtx {
        asset_manager: &mut asset_manager,
        canvas: &mut canvas,
        ui: &ui,
    };

    'running: loop {
        input_system(&mut command_queue, &mut event_pump);

        if command_queue.iter().any(|cmd| matches!(cmd, Command::Quit)) {
            break 'running;
        }

        selection_system(
            &command_queue,
            &mut event_queue,
            &mut game.state,
            &game.components,
        );
        placement_system(
            &command_queue,
            &mut event_queue,
            &mut game.state,
            &mut game.components,
        );
        rule_system(&mut flip_queue, &game.state, &game.components, &card_db);
        flip_system(&mut event_queue, &flip_queue, &mut game.components.owner);
        win_system(&mut event_queue, &game.state.phase, &game.components);
        render_system(&mut render_ctx, &game.state, &game.components, &card_db)?;

        director_system(&event_queue, &mut game.state, &game.components.position);

        command_queue.clear();
        event_queue.clear();
        flip_queue.clear();
    }

    Ok(())
}
