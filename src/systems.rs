use crate::{
    data::{CardDb, Stats},
    game::{self, Components, Entity, GRID_SIZE, Phase, Player, Position},
    query::{get_card_view, get_owned_entity, get_placed_entity, hand_size},
    render::render_card,
    sdl::AssetManager,
    ui::{Theme, UI},
};
use sdl2::{EventPump, keyboard::Keycode, pixels::Color, render::Canvas, video::Window};
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
        render_card(canvas, entity, ui, asset_manager, components, card_db)?;
    }

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
