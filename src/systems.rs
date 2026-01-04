use crate::{
    data::{CardDb, Stats},
    game::{self, Components, Entity, GRID_SIZE, Phase, Player, Position},
    query::{get_card_view, get_owned_entity, get_placed_entity, hand_size},
    sdl::{AssetLibrary, AssetManager},
    ui::{Theme, UI},
};
use sdl2::{
    EventPump, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};
use std::collections::VecDeque;

#[rustfmt::skip]
pub fn input_system(events: &mut VecDeque<game::Event>, phase: &Phase, event_pump: &mut EventPump) {
    for sdl_evt in event_pump.poll_iter() {
        match (phase, sdl_evt) {
            (_, sdl2::event::Event::Quit { .. }) => events.push_back(game::Event::Quit),

            (Phase::SelectCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Down), .. }) => events.push_back(game::Event::SelectCursorDown),
            (Phase::SelectCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Up), .. }) => events.push_back(game::Event::SelectCursorUp),
            (Phase::SelectCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Return), .. }) => events.push_back(game::Event::SelectCard),

            (Phase::PlaceCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Down), .. }) =>   events.push_back(game::Event::PlaceCursorDown),
            (Phase::PlaceCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Left), .. }) =>   events.push_back(game::Event::PlaceCursorLeft),
            (Phase::PlaceCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Right), .. }) =>  events.push_back(game::Event::PlaceCursorRight),
            (Phase::PlaceCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Up), .. }) =>     events.push_back(game::Event::PlaceCursorUp),
            (Phase::PlaceCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Return), .. }) => events.push_back(game::Event::PlaceCard),
            (Phase::PlaceCard, sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. }) => events.push_back(game::Event::CardDeselected),

            _ => {}
        }
    }
}

pub fn selection_system(
    events: &mut VecDeque<game::Event>,
    phase: &Phase,
    turn: &Option<Player>,
    cursor: &mut Option<Position>,
    selection: &mut Option<Entity>,
    components: &Components,
) {
    if !matches!(phase, Phase::SelectCard) {
        return;
    }

    let Some(player) = turn else {
        return;
    };

    let Components {
        owner, position, ..
    } = components;

    let maxlen = hand_size(player, owner, position);

    #[cfg_attr(any(), rustfmt::skip)]
    for evt in events.iter() {
        match (evt, cursor.as_mut()) {
            (game::Event::SelectCursorDown, Some(Position::Hand(j))) => *j = (*j + 1) % maxlen,
            (game::Event::SelectCursorUp, Some(Position::Hand(j))) => *j = ((*j as isize - 1) as usize).rem_euclid(maxlen),
            (game::Event::SelectCard, Some(Position::Hand(j))) => {
                *selection = get_owned_entity(player, &Position::Hand(*j), owner, position);
            }

            _ => {}
        }
    }

    if selection.is_some() {
        eprintln!("Selection -> CardSelected");
        events.push_back(game::Event::CardSelected);
    }
}

pub fn placement_system(
    events: &mut VecDeque<game::Event>,
    phase: &Phase,
    cursor: &mut Option<Position>,
    active_entity: &Option<Entity>,
    owners: &[Option<Player>],
    positions: &mut [Option<Position>],
) {
    if !matches!(phase, Phase::PlaceCard) {
        return;
    }

    let Some(selected_entity) = active_entity else {
        return;
    };

    let mut place_dst: Option<Position> = None;
    #[cfg_attr(any(), rustfmt::skip)]
    for evt in events.iter() {
        match (evt, cursor.as_mut()) {
            (game::Event::PlaceCursorDown, Some(Position::Board(_, y))) => *y = (*y + 1) % GRID_SIZE,
            (game::Event::PlaceCursorLeft, Some(Position::Board(x, _))) => *x = ((*x as isize - 1) as usize).rem_euclid(GRID_SIZE),
            (game::Event::PlaceCursorRight, Some(Position::Board(x, _))) => *x = (*x + 1) % GRID_SIZE,
            (game::Event::PlaceCursorUp, Some(Position::Board(_, y))) => *y = ((*y as isize - 1) as usize).rem_euclid(GRID_SIZE),
            (game::Event::PlaceCard, Some(Position::Board(x, y))) => {
                let position = Position::Board(*x, *y);
                // the destination cell is not occupied
                if get_placed_entity(&position, positions).is_none() {
                    place_dst = Some(position);
                }
            }
            //(game::Event::CardDeselected, _) => return,

            _ => {}
        }
    }

    // get entity's current hand position so that every other hand card can be shifted if necessary
    // replace position component
    // shift hand that has position > saved
    // fire event placed
    if let Some(position) = place_dst {
        let Some(Position::Hand(selected_hand_idx)) = positions[*selected_entity] else {
            return;
        };

        positions[*selected_entity] = Some(position);
        let player = &owners[*selected_entity];

        for entity in 0..owners.len() {
            if &owners[entity] != player {
                continue;
            }

            let Some(Position::Hand(k)) = positions[entity].as_mut() else {
                continue;
            };

            if *k > selected_hand_idx {
                *k -= 1;
            }
        }

        events.push_back(game::Event::CardPlaced);
    }
}

pub fn rule_system(
    events: &mut VecDeque<game::Event>,
    phase: &Phase,
    active_entity: &Option<Entity>,
    components: &Components,
    card_db: &CardDb,
) {
    if !matches!(phase, Phase::CheckNeighbors) {
        return;
    }

    let Some(placed_entity) = active_entity else {
        return;
    };

    let Some(placed_card) = get_card_view(*placed_entity, components, card_db) else {
        return;
    };

    let &Position::Board(placed_x, placed_y) = placed_card.position else {
        return;
    };

    let &Stats { top, rgt, btm, lft } = placed_card.stats;

    struct BattleCheck {
        in_bounds: bool,
        pos: Position,
        atk_stat: u8,
        def_stat_fn: fn(&Stats) -> u8,
    }

    let checks = [
        BattleCheck {
            in_bounds: placed_x > 0,
            pos: Position::Board((placed_x as isize - 1) as usize, placed_y),
            atk_stat: lft,
            def_stat_fn: |s| s.rgt,
        },
        BattleCheck {
            in_bounds: placed_x < 2,
            pos: Position::Board(placed_x + 1, placed_y),
            atk_stat: rgt,
            def_stat_fn: |s| s.lft,
        },
        BattleCheck {
            in_bounds: placed_y > 0,
            pos: Position::Board(placed_x, (placed_y as isize - 1) as usize),
            atk_stat: top,
            def_stat_fn: |s| s.btm,
        },
        BattleCheck {
            in_bounds: placed_y < 2,
            pos: Position::Board(placed_x, placed_y + 1),
            atk_stat: btm,
            def_stat_fn: |s| s.top,
        },
    ];

    for check in checks {
        if check.in_bounds {
            let Some(neighbor_entity) = get_placed_entity(&check.pos, &components.position) else {
                continue;
            };
            let Some(neighbor_card) = get_card_view(neighbor_entity, components, card_db) else {
                continue;
            };
            if placed_card.owner == neighbor_card.owner {
                continue;
            }
            if check.atk_stat > (check.def_stat_fn)(neighbor_card.stats) {
                events.push_back(game::Event::RuleFlip(neighbor_entity));
            }
        }
    }
}

pub fn flip_system(events: &VecDeque<game::Event>, owners: &mut [Option<Player>]) {
    for event in events {
        if let game::Event::RuleFlip(entity) = event
            && let Some(player) = owners[*entity].as_mut()
        {
            *player = !*player;
        }
    }
}

pub fn win_system(events: &mut VecDeque<game::Event>, phase: &Phase, components: &Components) {
    if !matches!(phase, Phase::TurnEnd) {
        return;
    }

    let placed_count = components
        .position
        .iter()
        .filter(|&pos| matches!(pos, Some(Position::Board(_, _))))
        .count();

    if placed_count < 9 {
        return;
    }

    let p1_score = components
        .owner
        .iter()
        .filter(|&player| matches!(player, Some(Player::P1)))
        .count();
    let p2_score = components
        .owner
        .iter()
        .filter(|&player| matches!(player, Some(Player::P2)))
        .count();

    if p1_score == p2_score {
        events.push_back(game::Event::DrawGame);
    } else if p1_score > p2_score {
        events.push_back(game::Event::PlayerWins(Player::P1));
    } else {
        events.push_back(game::Event::PlayerWins(Player::P2));
    }
}

pub fn render_system(
    canvas: &mut Canvas<Window>,
    ui: &UI,
    assets: &mut AssetLibrary,
    asset_manager: &mut AssetManager,
    turn: &Option<Player>,
    cursor: &Option<Position>,
    components: &Components,
    card_db: &CardDb,
) -> Result<(), String> {
    let Theme { bg, fg } = ui.palette.mono;

    canvas.set_draw_color(bg);
    canvas.clear();

    // render board
    {
        let s_cell = asset_manager.get_sprite("cell").unwrap();
        let t_cell = asset_manager.get_texture_mut(s_cell.texture_id).unwrap();
        t_cell.set_color_mod(fg.r, fg.g, fg.b);
        for rect in ui.layout.board {
            canvas.copy(t_cell, s_cell.region, rect)?;
        }
        t_cell.set_color_mod(255, 255, 255);
    }

    // render cards
    for entity in 0..10 {
        #[rustfmt::skip]
        let Some(card_view) = get_card_view(entity, components, card_db) else { continue };

        let rect = match (card_view.owner, card_view.position) {
            (Player::P1, Position::Hand(j)) => ui.layout.hand.p1[*j],
            (Player::P2, Position::Hand(j)) => ui.layout.hand.p2[*j],
            (_, Position::Board(x, y)) => ui.layout.board[*y * 3 + *x],
        };

        {
            let s_card_bg = asset_manager.get_sprite("card-bg").unwrap();
            let t_card_bg = asset_manager.get_texture_mut(s_card_bg.texture_id).unwrap();
            t_card_bg.set_color_mod(bg.r, bg.g, bg.b);
            canvas.copy(t_card_bg, s_card_bg.region, rect)?;
        }

        {
            let s_card_border = asset_manager.get_sprite("card-border").unwrap();
            let t_card_border = asset_manager
                .get_texture_mut(s_card_border.texture_id)
                .unwrap();
            t_card_border.set_color_mod(fg.r, fg.g, fg.b);
            canvas.copy(t_card_border, s_card_border.region, rect)?;
        }

        {
            let sprite = match card_view.owner {
                Player::P1 => "card-body-light",
                Player::P2 => "card-border-dark",
            };
            let s_card_border = asset_manager.get_sprite(sprite).unwrap();
            let t_card_border = asset_manager
                .get_texture_mut(s_card_border.texture_id)
                .unwrap();
            t_card_border.set_color_mod(fg.r, fg.g, fg.b);
            canvas.copy(t_card_border, s_card_border.region, rect)?;
        }
    }

    //for entity in 0..10 {
    //    let Some(card_view) = get_card_view(entity, components, card_db) else {
    //        continue;
    //    };
    //    let rect = match (card_view.owner, card_view.position) {
    //        (Player::P1, Position::Hand(j)) => ui.layout.hand.p1[*j],
    //        (Player::P2, Position::Hand(j)) => ui.layout.hand.p2[*j],
    //        (_, Position::Board(x, y)) => ui.layout.board[*y * 3 + *x],
    //    };
    //    let color = match card_view.owner {
    //        Player::P1 => ui.palette.wireframe.p1,
    //        Player::P2 => ui.palette.wireframe.p2,
    //    };

    //    {
    //        let card_texture = assets.get_texture_mut("card");
    //        card_texture.set_color_mod(color.r, color.g, color.b);
    //        canvas.copy(card_texture, None, rect)?;
    //    }

    //    let stats = [
    //        (card_view.stats.top, ui.layout.card.stats.top),
    //        (card_view.stats.lft, ui.layout.card.stats.lft),
    //        (card_view.stats.rgt, ui.layout.card.stats.rgt),
    //        (card_view.stats.btm, ui.layout.card.stats.btm),
    //    ];

    //    let font_texture = assets.get_texture("font");
    //    for (value, dst) in stats {
    //        canvas.copy(
    //            font_texture,
    //            Rect::new(value as i32 * 9, 7, 9, 11),
    //            Rect::new(rect.x() + dst.x(), rect.y() + dst.y(), 9, 11),
    //        )?;
    //    }
    //    // render stats
    //}

    if let Some(cursor) = cursor {
        canvas.set_draw_color(Color::RGB(255, 255, 0));
        if let Some(rect) = match (turn, cursor) {
            (Some(Player::P1), Position::Hand(j)) => Some(ui.layout.hand.p1[*j]),
            (Some(Player::P2), Position::Hand(j)) => Some(ui.layout.hand.p2[*j]),
            (_, Position::Board(x, y)) => Some(ui.layout.board[*y * 3 + *x]),
            _ => None,
        } {
            canvas.draw_rect(rect.right_shifted(10).bottom_shifted(10))?;
        }
    }

    canvas.present();

    Ok(())
}

/// Returns whether the game is running or not.
pub fn director_system(
    events: &VecDeque<game::Event>,
    phase: &mut Phase,
    turn: &mut Option<Player>,
    cursor: &mut Option<Position>,
    active_entity: &mut Option<Entity>,
    position: &[Option<Position>],
) -> bool {
    if events.iter().any(|e| matches!(e, game::Event::Quit)) {
        return false;
    }

    match phase {
        Phase::GameStart => {
            *phase = Phase::TurnStart;
            *turn = Some(Player::P1);
        }

        Phase::TurnStart => {
            *phase = Phase::SelectCard;
            *cursor = Some(Position::Hand(0));
        }

        Phase::SelectCard => {
            if events
                .iter()
                .any(|e| matches!(e, game::Event::CardSelected))
            {
                *phase = Phase::PlaceCard;
                *cursor = Some(Position::Board(1, 1));
            }
        }

        Phase::PlaceCard => {
            #[cfg_attr(any(), rustfmt::skip)]
            let deselected = events.iter().any(|e| matches!(e, game::Event::CardDeselected));
            let placed = events.iter().any(|e| matches!(e, game::Event::CardPlaced));

            if deselected {
                let hand_index = active_entity
                    .take()
                    .and_then(|entity| position[entity].as_ref())
                    .map_or(0, |pos| match pos {
                        Position::Hand(j) => *j,
                        _ => 0,
                    });

                *phase = Phase::SelectCard;
                *cursor = Some(Position::Hand(hand_index));
            } else if placed {
                *phase = Phase::CheckNeighbors;
                *cursor = None;
            }
        }

        Phase::CheckNeighbors => *phase = Phase::TurnEnd,

        Phase::TurnEnd => {
            *active_entity = None;

            if events
                .iter()
                .any(|e| matches!(e, game::Event::DrawGame | game::Event::PlayerWins(_)))
            {
                eprintln!("TurnEnd -> GameOver");
                *phase = Phase::GameOver;
                *turn = None;
            } else {
                eprintln!("TurnEnd -> SwitchPlayer");
                *phase = Phase::SwitchPlayer;
            };
        }

        Phase::SwitchPlayer => {
            if let Some(player) = turn.as_mut() {
                *player = !*player;
                *phase = Phase::TurnStart;
            }
        }

        Phase::GameOver => {}
    }

    true
}

//use std::collections::VecDeque;
//
//use sdl2::{
//    EventPump,
//    event::Event,
//    keyboard::Keycode,
//    pixels::Color,
//    rect::Rect,
//    render::{Canvas, Texture},
//    video::Window,
//};
//
//use crate::{
//    data::{CardDb, CardStats},
//    event::GameEvent,
//    game::{
//        Board, CardView, Components, Cursor, Entity, GamePhase, InteractionCtx, Player, Resources,
//        c2i, card_at,
//    },
//    query::{can_place_card, get_hand_entity, hand_size},
//    sdl::AssetLibrary,
//    ui::UI,
//};
//
///// Update board state, hand state and card position, committing card placement.
/////
///// This is important because resources (board and hand) and components (card position) must be
///// kept in sync.
//pub fn board_state_system(
//    events: &mut VecDeque<GameEvent>,
//    turn: Option<Player>,
//    interaction: &InteractionCtx,
//    positions: &mut [Option<Cursor>],
//    owners: &[Option<Player>],
//) {
//    if !matches!(interaction.phase, GamePhase::PlaceCard)
//        || turn.is_none()
//        || !matches!(interaction.selection, Some(Cursor::Hand(_)))
//        || !events
//            .iter()
//            .any(|event| matches!(event, GameEvent::PlaceCard))
//    {
//        return;
//    }
//
//    let Some(Cursor::Hand(placed_index)) = interaction.selection else {
//        return;
//    };
//    let Some(Cursor::Board(x, y)) = interaction.cursor else {
//        return;
//    };
//    let Some(entity) = (0..10)
//        .find(|&entity| owners[entity] == turn && positions[entity] == interaction.selection)
//    else {
//        return;
//    };
//
//    for j in 0..10 {
//        if owners[j] != turn {
//            continue;
//        }
//
//        if j == entity {
//            positions[entity] = Some(Cursor::Board(x, y));
//            continue;
//        }
//
//        if let Some(Cursor::Hand(i)) = positions[j].as_mut()
//            && *i > placed_index
//        {
//            *i -= 1;
//        }
//    }
//
//    events.push_back(GameEvent::CardPlaced(entity));
//}
//
//pub fn director_system(
//    events: &VecDeque<GameEvent>,
//    turn: &mut Option<Player>,
//    interaction: &mut InteractionCtx,
//) {
//    match interaction.phase {
//        GamePhase::GameStart => {
//            *turn = Some(Player::P1); // FIXME this should be random
//            interaction.phase = GamePhase::TurnStart;
//        }
//        GamePhase::TurnStart => interaction.phase = GamePhase::SelectCard,
//        GamePhase::SelectCard => {
//            if events
//                .iter()
//                .any(|event| matches!(event, GameEvent::CursorSelect))
//            {
//                interaction.phase = GamePhase::PlaceCard;
//            }
//        }
//        GamePhase::PlaceCard => {
//            for event in events.iter() {
//                match event {
//                    GameEvent::CursorBack => interaction.phase = GamePhase::SelectCard,
//                    GameEvent::PlaceCard => interaction.phase = GamePhase::CheckNeighbors,
//                    _ => {}
//                }
//            }
//        }
//        GamePhase::CheckNeighbors => {
//            interaction.phase = GamePhase::TurnEnd;
//            interaction.cursor = None;
//            interaction.selection = None;
//        }
//        GamePhase::TurnEnd => interaction.phase = GamePhase::SwitchPlayer,
//        GamePhase::SwitchPlayer => {
//            if let Some(player) = turn.as_mut() {
//                *player = !*player;
//            }
//            interaction.phase = GamePhase::TurnStart;
//        }
//        _ => todo!(),
//    }
//}
//
//pub fn flip_system(
//    events: &VecDeque<GameEvent>,
//    interaction: &InteractionCtx,
//    owner: &mut [Option<Player>],
//) {
//    if !matches!(interaction.phase, GamePhase::CheckNeighbors) {
//        return;
//    }
//
//    for event in events {
//        let &GameEvent::FlipCard(entity) = event else {
//            continue;
//        };
//
//        let Some(player) = owner[entity].as_mut() else {
//            continue;
//        };
//
//        *player = !*player;
//    }
//}
//
///// Translate SDL events into game semantic events.
//pub fn input_system(event_pump: &mut EventPump, event_queue: &mut VecDeque<GameEvent>) {
//    for event in event_pump.poll_iter() {
//        #[cfg_attr(any(), rustfmt::skip)]
//        match event {
//            Event::Quit { .. } => event_queue.push_back(GameEvent::Quit),
//
//            Event::KeyDown { keycode: Some(Keycode::Down), .. } => event_queue.push_back(GameEvent::CursorDown),
//            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => event_queue.push_back(GameEvent::CursorBack),
//            Event::KeyDown { keycode: Some(Keycode::Left), .. } => event_queue.push_back(GameEvent::CursorLeft),
//            Event::KeyDown { keycode: Some(Keycode::Return), .. } => event_queue.push_back(GameEvent::CursorSelect),
//            Event::KeyDown { keycode: Some(Keycode::Right), .. } => event_queue.push_back(GameEvent::CursorRight),
//            Event::KeyDown { keycode: Some(Keycode::Up), .. } => event_queue.push_back(GameEvent::CursorUp),
//
//            _ => {}
//        }
//    }
//}
//
///// Handle selected card placement on game board.
//pub fn placement_system(
//    events: &mut VecDeque<GameEvent>,
//    interaction: &mut InteractionCtx,
//    positions: &[Option<Cursor>],
//) {
//    if !matches!(interaction.phase, GamePhase::PlaceCard) {
//        return;
//    }
//
//    if interaction.cursor.is_none() {
//        interaction.cursor = Some(Cursor::Board(1, 1));
//    }
//
//    let mut place = false;
//    for event in events.iter() {
//        match event {
//            GameEvent::CursorBack => interaction.undo_select_card(),
//            GameEvent::CursorDown => interaction.move_board_cursor_down(),
//            GameEvent::CursorLeft => interaction.move_board_cursor_left(),
//            GameEvent::CursorRight => interaction.move_board_cursor_right(),
//            GameEvent::CursorUp => interaction.move_board_cursor_up(),
//            GameEvent::CursorSelect => {
//                if can_place_card(interaction, positions) {
//                    place = true;
//                    break; // there is only one (cursor, selection) pair
//                }
//            }
//            _ => {}
//        }
//    }
//
//    if place {
//        events.push_back(GameEvent::PlaceCard);
//    }
//}
//
//// NOTE possibly squash with director system
///// Check whether a quit event was pushed and in case make the game quit.
//pub fn quit_system(event_queue: &VecDeque<GameEvent>) -> bool {
//    event_queue
//        .iter()
//        .any(|event| matches!(event, GameEvent::Quit))
//}
//
///// Neighbor map
/////
///// ```txt
/////     board          vector
///// -------------    ---------
///// 0,0  1,0  2,0     0  1  2
///// 0,1  1,1  2,1     3  4  5
///// 0,2  1,2  2,2     6  7  8
///// ```
/////
///// NOTE events are not currently used but card flip might require playing sound, so events might
///// be introduced later.
//pub fn rule_system(
//    events: &mut VecDeque<GameEvent>,
//    interaction: &InteractionCtx,
//    components: &Components,
//    card_db: &CardDb,
//) {
//    if !matches!(interaction.phase, GamePhase::CheckNeighbors) {
//        return;
//    }
//
//    let Some(entity) = events.iter().find_map(|&event| {
//        if let GameEvent::CardPlaced(entity) = event {
//            Some(entity)
//        } else {
//            None
//        }
//    }) else {
//        return;
//    };
//
//    let Some((x, y, placed_card)) = interaction.placed_card(entity, components, card_db) else {
//        return;
//    };
//
//    struct BattleDir {
//        in_bounds: bool,
//        offset: isize,
//        atk: u8,
//        /// Defense stat getter.
//        def_fn: fn(&CardStats) -> u8,
//    }
//
//    #[rustfmt::skip]
//    let checks: [BattleDir; 4] = [
//        BattleDir { in_bounds: y > 0, offset: -3, atk: placed_card.stats.top, def_fn: |neigh| neigh.btm},
//        BattleDir { in_bounds: x > 0, offset: -1, atk: placed_card.stats.lft, def_fn: |neigh| neigh.rgt},
//        BattleDir { in_bounds: x < 2, offset:  1, atk: placed_card.stats.rgt, def_fn: |neigh| neigh.lft},
//        BattleDir { in_bounds: y < 2, offset:  3, atk: placed_card.stats.btm, def_fn: |neigh| neigh.top},
//    ];
//
//    let mut cards_to_flip: Vec<Entity> = Vec::with_capacity(8);
//
//    for check in checks {
//        if !check.in_bounds {
//            continue;
//        }
//
//        let placed_idx = Board::into_index(x, y);
//        let neighbor_idx = (placed_idx as isize + check.offset) as usize;
//        let Some(neighbor_entity) = board.board.state[neighbor_idx] else {
//            continue;
//        };
//        if let Some(neighbor) = card_at(neighbor_entity, components, card_db)
//            && placed_card.owner != neighbor.owner
//            && check.atk > (check.def_fn)(neighbor.stats)
//        {
//            cards_to_flip.push(neighbor.entity);
//        }
//    }
//
//    for entity in cards_to_flip {
//        events.push_back(GameEvent::FlipCard(entity));
//    }
//}
//
///// Handle card selection from player's hand.
//pub fn selection_system(
//    events: &mut VecDeque<GameEvent>,
//    turn: Option<Player>,
//    interaction: &mut InteractionCtx,
//    components: &Components,
//) {
//    if !matches!(interaction.phase, GamePhase::SelectCard) {
//        return;
//    }
//
//    let Some(player) = turn else {
//        return;
//    };
//
//    if interaction.cursor.is_none() {
//        interaction.cursor = Some(Cursor::Hand(0));
//    }
//
//    let hand_size = hand_size(player, &components.owner, &components.position) as u8;
//
//    let mut entity: Option<Entity> = None;
//    for event in events.iter() {
//        match event {
//            GameEvent::CursorDown => interaction.move_hand_cursor_down(hand_size),
//            GameEvent::CursorUp => interaction.move_hand_cursor_up(hand_size),
//            GameEvent::CursorSelect => {
//                if let Some(hand_position) = interaction.select_card() {
//                    entity = get_hand_entity(
//                        player,
//                        &hand_position,
//                        &components.owner,
//                        &components.position,
//                    );
//                }
//            }
//            _ => {}
//        }
//    }
//
//    if let Some(entity) = entity {
//        events.push_back(GameEvent::CardSelected(entity));
//    }
//}
//
//// ================================= Render ===================================
//
//pub fn render_system(
//    canvas: &mut Canvas<Window>,
//    ui: &UI,
//    assets: &AssetLibrary,
//    resources: &Resources,
//    components: &Components,
//    card_db: &CardDb,
//) -> Result<(), String> {
//    const MAX_HAND_SIZE: usize = 5; // FIXME move elsewhere
//    let layout = &ui.layout;
//
//    let font_texture = assets.get_texture("font");
//
//    canvas.set_draw_color(Color::RGB(0, 0, 0));
//    canvas.clear();
//
//    draw_board(canvas, ui)?;
//
//    for entity in 0..=9 {
//        let Some(card_view) = card_at(entity, components, card_db) else {
//            continue;
//        };
//        draw_card(canvas, &card_view, resources, ui, font_texture)?;
//    }
//
//    // render cursor
//    // TODO
//    //match (resources.board.turn, &resources.interaction.cursor) {
//    //    (Some(Player::P1), Some(Cursor::Hand(j))) => {
//    //        let start = MAX_HAND_SIZE - resources.board.hand.p1.len();
//    //        let j = start + *j as usize;
//    //        let card = layout.hand.p1[j];
//    //        canvas.set_draw_color(Color::RGB(255, 255, 0));
//    //        canvas.draw_rect(Rect::new(
//    //            card.x() + card.width() as i32 + 35,
//    //            card.y() + (card.height() / 2) as i32 - 12,
//    //            50,
//    //            20,
//    //        ))?;
//    //    }
//
//    //    (Some(Player::P2), Some(Cursor::Hand(j))) => {
//    //        let start = MAX_HAND_SIZE - resources.board.hand.p2.len();
//    //        let j = start + *j as usize;
//    //        let card = layout.hand.p2[j];
//    //        canvas.set_draw_color(Color::RGB(255, 255, 0));
//    //        canvas.draw_rect(Rect::new(
//    //            card.x() - 50 - 35,
//    //            card.y() + (card.height() / 2) as i32 - 12,
//    //            50,
//    //            20,
//    //        ))?;
//    //    }
//
//    //    (Some(_), Some(Cursor::Board(x, y))) => {
//    //        // TODO flip cursor based on P1 | P2
//    //        let j = (y * 3) + x; // FIXME magic number
//    //        let cell = layout.board[j as usize];
//    //        let center = cell.center();
//    //        canvas.set_draw_color(Color::RGB(255, 255, 0));
//    //        canvas.draw_rect(Rect::new(center.x(), center.y(), 50, 20))?;
//    //    }
//
//    //    _ => {}
//    //}
//
//    // render selection
//
//    canvas.present();
//
//    Ok(())
//}
//
///// Draw 3x3 board.
//fn draw_board(canvas: &mut Canvas<Window>, ui: &UI) -> Result<(), String> {
//    let UI { layout, palette } = ui;
//
//    canvas.set_draw_color(palette.wireframe.board);
//
//    for rect in layout.board.iter() {
//        canvas.draw_rect(*rect)?;
//    }
//
//    Ok(())
//}
//
//// FIXME move this in layout or in a struct that makes sense.
//const MAX_HAND_SIZE: usize = 5;
//
//fn draw_card(
//    canvas: &mut Canvas<Window>,
//    card: &CardView,
//    resources: &Resources,
//    ui: &UI,
//    font: &Texture,
//) -> Result<(), String> {
//    let UI { layout, palette } = ui;
//
//    // FIXME -> no board
//    // let start_p1 = MAX_HAND_SIZE - resources.board.hand.p1.len();
//    // let start_p2 = MAX_HAND_SIZE - resources.board.hand.p2.len();
//
//    // TODO selected shift by 30 px
//    // selected computation is tedious because we need to combine
//    // cursor -> hand -> index ->
//    //           hand -> player-> entity
//
//    let rect = match (card.position, card.owner) {
//        (&Cursor::Hand(j), Player::P1) => layout.hand.p1[start_p1 + j as usize],
//        (&Cursor::Hand(j), Player::P2) => layout.hand.p2[start_p2 + j as usize],
//        (&Cursor::Board(x, y), _) => layout.board[c2i(x, y) as usize],
//    };
//    let color = match card.owner {
//        Player::P1 => palette.wireframe.p1,
//        Player::P2 => palette.wireframe.p2,
//    };
//
//    canvas.set_draw_color(color);
//    canvas.draw_rect(rect)?;
//
//    //
//    canvas.set_draw_color(Color::RGB(0, 0, 0));
//    let mut fill = rect.bottom_shifted(1).right_shifted(1);
//    fill.resize(
//        (rect.width() as i32 - 2) as u32,
//        (rect.height() as i32 - 2) as u32,
//    );
//    canvas.fill_rect(fill)?;
//    //
//
//    canvas.set_draw_color(color);
//    let padding = layout.card.padding as i32;
//    let mut fill = rect.bottom_shifted(padding).right_shifted(padding);
//    fill.resize(
//        (rect.width() as i32 - 2 * padding) as u32,
//        (rect.height() as i32 - 2 * padding) as u32,
//    );
//    canvas.fill_rect(fill)?;
//
//    let stats = [
//        (card.stats.top, layout.card.stats.top),
//        (card.stats.lft, layout.card.stats.lft),
//        (card.stats.rgt, layout.card.stats.rgt),
//        (card.stats.btm, layout.card.stats.btm),
//    ];
//
//    for (value, geom) in stats {
//        canvas.copy(
//            font,
//            Rect::new((9 * value) as i32, 7, 9, 11),
//            Rect::new(
//                rect.x() + geom.x(),
//                rect.y() + geom.y(),
//                geom.width(),
//                geom.height(),
//            ),
//        )?;
//    }
//
//    Ok(())
//}
//
//// === render util ===
//fn render_stats(
//    canvas: &mut Canvas<Window>,
//    stats: &CardStats,
//    texture: &Texture,
//    rect: &Rect,
//) -> Result<(), String> {
//    // top
//    canvas.copy(
//        texture,
//        Rect::new((9 * stats.top) as i32, 7, 9, 11),
//        Rect::new(rect.x() + 8, rect.y() + 8, 9, 11),
//    )?;
//    // right
//    canvas.copy(
//        texture,
//        Rect::new((9 * stats.rgt) as i32, 7, 9, 11),
//        Rect::new(rect.x() + 8 + 5, rect.y() + 8 + 11, 9, 11),
//    )?;
//    // left
//    canvas.copy(
//        texture,
//        Rect::new((9 * stats.lft) as i32, 7, 9, 11),
//        Rect::new(rect.x() + 8 - 5, rect.y() + 8 + 11, 9, 11),
//    )?;
//    // bottom
//    canvas.copy(
//        texture,
//        Rect::new((9 * stats.btm) as i32, 7, 9, 11),
//        Rect::new(rect.x() + 8, rect.y() + 8 + 11 + 11, 9, 11),
//    )?;
//    Ok(())
//}
//// === render util === (end)
