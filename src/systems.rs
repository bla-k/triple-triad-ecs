use crate::{
    data::{CardDb, Stats},
    event::{Command, Direction, GameEvent, MatchResult},
    game::{Components, Entity, Phase, Player, Position, SessionState},
    query::{get_card_view, get_owned_entity, get_placed_entity, hand_size},
    render::{RenderCtx, render_board, render_card},
    ui::{Layout, Theme},
};
use sdl2::{EventPump, keyboard::Keycode, rect::Rect};
use std::collections::VecDeque;

pub fn input_system(commands: &mut VecDeque<Command>, event_pump: &mut EventPump) {
    use sdl2::event::Event;

    for evt in event_pump.poll_iter() {
        if let Some(command) = match evt {
            Event::Quit { .. } => Some(Command::Quit),

            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => Some(Command::MoveCursor(Direction::Down)),

            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => Some(Command::MoveCursor(Direction::Left)),

            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => Some(Command::MoveCursor(Direction::Right)),

            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => Some(Command::MoveCursor(Direction::Up)),

            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => Some(Command::Cancel),

            Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => Some(Command::Confirm),

            _ => None,
        } {
            commands.push_back(command);
        };
    }
}

pub fn selection_system(
    commands: &VecDeque<Command>,
    game_events: &mut VecDeque<GameEvent>,
    state: &mut SessionState,
    components: &Components,
) {
    if !matches!(state.phase, Phase::SelectCard) {
        return;
    }

    let Some(player) = state.turn else {
        return;
    };

    let Components {
        owner, position, ..
    } = components;

    let maxlen = hand_size(player, owner, position);

    let mut card_selected = false;
    for command in commands {
        match (command, state.cursor.as_mut()) {
            (Command::MoveCursor(Direction::Down), Some(Position::Hand(j))) => {
                *j = (*j + 1) % maxlen
            }

            (Command::MoveCursor(Direction::Up), Some(Position::Hand(j))) => {
                *j = (*j as isize - 1).rem_euclid(maxlen as isize) as usize
            }

            (Command::Confirm, Some(Position::Hand(_))) => card_selected = true,

            _ => {}
        }
    }

    // always update selection by reading current cursor, so on player's turn start the preset
    // `Hand(0)` card appears selected
    if let Some(Position::Hand(j)) = state.cursor {
        state.active_entity = get_owned_entity(player, Position::Hand(j), owner, position);
    }

    if state.active_entity.is_some() && card_selected {
        game_events.push_back(GameEvent::CardSelected);
    }
}

pub fn placement_system(
    commands: &VecDeque<Command>,
    game_events: &mut VecDeque<GameEvent>,
    state: &mut SessionState,
    components: &mut Components,
) {
    if !matches!(state.phase, Phase::PlaceCard) {
        return;
    }

    let Some(selected_entity) = state.active_entity else {
        return;
    };

    let mut place_dst: Option<Position> = None;
    for command in commands.iter() {
        match (command, state.cursor.as_mut()) {
            (Command::MoveCursor(Direction::Down), Some(Position::Board(_, y))) => {
                *y = (*y + 1) % Layout::GRID_SIZE
            }
            (Command::MoveCursor(Direction::Left), Some(Position::Board(x, _))) => {
                *x = (*x as isize - 1).rem_euclid(Layout::GRID_SIZE as isize) as usize
            }
            (Command::MoveCursor(Direction::Right), Some(Position::Board(x, _))) => {
                *x = (*x + 1) % Layout::GRID_SIZE
            }
            (Command::MoveCursor(Direction::Up), Some(Position::Board(_, y))) => {
                *y = (*y as isize - 1).rem_euclid(Layout::GRID_SIZE as isize) as usize
            }
            (Command::Cancel, Some(Position::Board(_, _))) => {
                game_events.push_back(GameEvent::CardDeselected)
            }
            (Command::Confirm, Some(Position::Board(x, y))) => {
                let position = Position::Board(*x, *y);
                // the destination cell is not occupied
                if get_placed_entity(position, &components.position).is_none() {
                    place_dst = Some(position);
                }
            }

            _ => {}
        }
    }

    // get entity's current hand position so that every other hand card can be shifted if necessary
    // replace position component
    // shift hand that has position > saved
    // fire event placed
    if let Some(position) = place_dst {
        let Some(Position::Hand(selected_hand_idx)) = components.position[selected_entity.id()]
        else {
            return;
        };

        components.position[selected_entity.id()] = Some(position);
        let player = &components.owner[selected_entity.id()];

        for entity in 0..components.owner.len() {
            if &components.owner[entity] != player {
                continue;
            }

            let Some(Position::Hand(k)) = components.position[entity].as_mut() else {
                continue;
            };

            if *k > selected_hand_idx {
                *k -= 1;
            }
        }

        game_events.push_back(GameEvent::CardPlaced);
    }
}

pub fn rule_system(
    game_events: &mut VecDeque<Entity>,
    state: &SessionState,
    components: &Components,
    card_db: &CardDb,
) {
    if !matches!(state.phase, Phase::CheckNeighbors) {
        return;
    }

    let Some(placed_entity) = state.active_entity else {
        return;
    };

    let Some(placed_card) = get_card_view(placed_entity, components, card_db) else {
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
            let Some(neighbor_entity) = get_placed_entity(check.pos, &components.position) else {
                continue;
            };
            let Some(neighbor_card) = get_card_view(neighbor_entity, components, card_db) else {
                continue;
            };
            if placed_card.owner == neighbor_card.owner {
                continue;
            }
            if check.atk_stat > (check.def_stat_fn)(neighbor_card.stats) {
                game_events.push_back(neighbor_entity);
            }
        }
    }
}

pub fn flip_system(
    events_out: &mut VecDeque<GameEvent>,
    events_in: &VecDeque<Entity>,
    owners: &mut [Option<Player>],
) {
    for entity in events_in {
        if let Some(player) = owners[entity.id()].as_mut() {
            *player = !*player;
            events_out.push_back(GameEvent::CardFlipped);
        }
    }
}

pub fn win_system(events_out: &mut VecDeque<GameEvent>, phase: &Phase, components: &Components) {
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
        events_out.push_back(GameEvent::MatchEnded(MatchResult::Draw));
    } else if p1_score > p2_score {
        events_out.push_back(GameEvent::MatchEnded(MatchResult::Winner(Player::P1)));
    } else {
        events_out.push_back(GameEvent::MatchEnded(MatchResult::Winner(Player::P2)));
    }
}

pub fn render_system(
    ctx: &mut RenderCtx,
    state: &SessionState,
    components: &Components,
    card_db: &CardDb,
) -> Result<(), String> {
    let Theme { bg, fg } = ctx.ui.palette.mono;

    ctx.canvas.set_draw_color(bg);
    ctx.canvas.clear();

    render_board(ctx)?;

    // render cards
    for j in 0..10 {
        render_card(ctx, Entity(j), state.active_entity, components, card_db)?;
    }

    // render cursor
    if let Some(cursor) = state.cursor {
        match (state.turn, cursor) {
            // cursor to the right of the card
            (Some(Player::P1), Position::Hand(j)) => {
                let s_cursor = ctx.asset_manager.get_sprite("cursor").unwrap();
                let t_cursor = ctx
                    .asset_manager
                    .get_texture_mut(s_cursor.texture_id)
                    .unwrap();
                t_cursor.set_color_mod(fg.r, fg.g, fg.b);

                let card_rect = ctx.ui.layout.hand.p1[j];
                let cursor_rect = Rect::new(
                    card_rect.x() + card_rect.width() as i32 + 24,
                    card_rect.y() + (card_rect.height() / 2) as i32
                        - (s_cursor.region.height() / 2) as i32,
                    s_cursor.region.width(),
                    s_cursor.region.height(),
                );

                ctx.canvas.copy(t_cursor, s_cursor.region, cursor_rect)?;

                t_cursor.set_color_mod(255, 255, 255);
            }

            // flipped cursor to the left of the card
            (Some(Player::P2), Position::Hand(j)) => {
                let s_cursor = ctx.asset_manager.get_sprite("cursor").unwrap();
                let t_cursor = ctx
                    .asset_manager
                    .get_texture_mut(s_cursor.texture_id)
                    .unwrap();
                t_cursor.set_color_mod(fg.r, fg.g, fg.b);

                let card_rect = ctx.ui.layout.hand.p2[j];
                let cursor_rect = Rect::new(
                    card_rect.x() - 34,
                    card_rect.y() + (card_rect.height() / 2) as i32
                        - (s_cursor.region.height() / 2) as i32,
                    s_cursor.region.width(),
                    s_cursor.region.height(),
                );

                ctx.canvas.copy_ex(
                    t_cursor,
                    s_cursor.region,
                    cursor_rect,
                    0.0,
                    None,
                    true,
                    false,
                )?;

                t_cursor.set_color_mod(255, 255, 255);
            }

            // cursor highlighting the center of the cell
            (_, Position::Board(x, y)) => {
                let j = y * 3 + x; // FIXME magic number
                let card_rect = ctx.ui.layout.board[j];
                let mut cursor = card_rect.left_shifted(8).top_shifted(8);
                cursor.resize(card_rect.width() + 16, card_rect.height() + 16);

                ctx.canvas.set_draw_color(fg);
                ctx.canvas.draw_rect(cursor)?;
            }
            _ => {}
        }
    }

    ctx.canvas.present();

    Ok(())
}

/// Returns whether the game is running or not.
pub fn director_system(
    events: &VecDeque<GameEvent>,
    state: &mut SessionState,
    position: &[Option<Position>],
) {
    match state.phase {
        Phase::GameStart => {
            state.phase = Phase::TurnStart;
            state.turn = Some(Player::P1);
        }

        Phase::TurnStart => {
            state.phase = Phase::SelectCard;
            state.cursor = Some(Position::Hand(0));
        }

        Phase::SelectCard => {
            if events.iter().any(|e| matches!(e, GameEvent::CardSelected)) {
                state.phase = Phase::PlaceCard;
                state.cursor = Some(Position::Board(1, 1));
            }
        }

        Phase::PlaceCard => {
            #[cfg_attr(any(), rustfmt::skip)]
            let deselected = events.iter().any(|e| matches!(e, GameEvent::CardDeselected));
            let placed = events.iter().any(|e| matches!(e, GameEvent::CardPlaced));

            if deselected {
                let hand_index = state
                    .active_entity
                    .take()
                    .and_then(|entity| position[entity.id()].as_ref())
                    .map_or(0, |pos| match pos {
                        Position::Hand(j) => *j,
                        _ => 0,
                    });

                state.phase = Phase::SelectCard;
                state.cursor = Some(Position::Hand(hand_index));
            } else if placed {
                state.phase = Phase::CheckNeighbors;
                state.cursor = None;
            }
        }

        Phase::CheckNeighbors => state.phase = Phase::TurnEnd,

        Phase::TurnEnd => {
            state.active_entity = None;

            if events.iter().any(|e| {
                matches!(
                    e,
                    GameEvent::MatchEnded(MatchResult::Draw)
                        | GameEvent::MatchEnded(MatchResult::Winner(_))
                )
            }) {
                state.phase = Phase::GameOver;
                state.turn = None;
            } else {
                state.phase = Phase::SwitchPlayer;
            };
        }

        Phase::SwitchPlayer => {
            if let Some(player) = state.turn.as_mut() {
                *player = !*player;
                state.phase = Phase::TurnStart;
            }
        }

        Phase::GameOver => {}
    }
}
