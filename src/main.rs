use sdl2::rect::Rect;
use triple_triad::{
    core::battle::{Battle, BattleSetup, Player},
    data::CardDb,
    event::{self, Command},
    render::RenderCtx,
    sdl::{AssetManager, BakeCardCfg, SdlSystems, Sprite},
    sys::rand::Rng,
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

    let event::Bus {
        mut commands,
        mut events,
        mut flips,
    } = event::Bus::default();

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

    let mut rng = Rng::init();
    //let mut rng = Rng::from_seed([
    //    0xe92c5a15dfffe2b9,
    //    0x1a12045ae731366f,
    //    0x3830d27519e55407,
    //    0x84f0f610b170b3e6,
    //]);

    println!("{}", rng);

    let battle_setup = BattleSetup {
        p1_hand: [
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
        ],
        p2_hand: [
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
            rng.next_bounded(CardDb::CARD_COUNT as u64) as usize,
        ],
    };

    let Battle {
        mut state,
        mut components,
        ..
    } = battle_setup.into();

    let mut render_ctx = RenderCtx {
        asset_manager: &mut asset_manager,
        canvas: &mut canvas,
        ui: &ui,
    };

    'running: loop {
        input_system(&mut commands, &mut event_pump);

        if commands.iter().any(|cmd| matches!(cmd, Command::Quit)) {
            break 'running;
        }

        selection_system(&commands, &mut events, &mut state, &components);
        placement_system(&commands, &mut events, &mut state, &mut components);
        rule_system(&mut flips, &state, &components, &card_db);
        flip_system(&mut events, &flips, &mut components.owner);
        win_system(&mut events, state, &components);
        render_system(&mut render_ctx, &state, &components, &card_db)?;

        director_system(&events, &mut state, &components.owner, &components.position);

        commands.clear();
        events.clear();
        flips.clear();
    }

    Ok(())
}
