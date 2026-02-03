use crate::{
    core::battle::Entity,
    data::{CardDb, Stats},
    event::{Command, Direction, GameEvent, MatchResult},
    game::{Components, MatchState, Player, Position, TurnPhase},
    query::{get_card_view, get_owned_entity, get_placed_entity, hand_size},
    render::{RenderCtx, render_board, render_card},
    rules::{wrap_decr, wrap_incr},
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
    state: &mut MatchState,
    components: &Components,
) {
    let MatchState::Turn {
        phase: TurnPhase::SelectCard { cursor, entity },
        player,
    } = state
    else {
        return;
    };

    let Components {
        owner, position, ..
    } = components;

    let maxlen = hand_size(*player, owner, position);

    let mut card_selected = false;
    for command in commands {
        match command {
            Command::MoveCursor(Direction::Down) => *cursor = wrap_incr(*cursor, maxlen),
            Command::MoveCursor(Direction::Up) => *cursor = wrap_decr(*cursor, maxlen),
            Command::Confirm => card_selected = true,
            _ => {}
        }
    }

    // always update selection by reading current cursor, so on player's turn start the preset
    // `Hand(0)` card appears selected
    *entity = get_owned_entity(*player, Position::Hand(*cursor), owner, position);

    if let Some(target) = entity
        && card_selected
    {
        game_events.push_back(GameEvent::CardSelected { target: *target });
    }
}

pub fn placement_system(
    commands: &VecDeque<Command>,
    game_events: &mut VecDeque<GameEvent>,
    mstate: &mut MatchState,
    components: &mut Components,
) {
    let MatchState::Turn {
        phase: TurnPhase::PlaceCard { cursor, entity },
        ..
    } = mstate
    else {
        return;
    };

    let mut place_dst: Option<Position> = None;
    for command in commands.iter() {
        match command {
            Command::MoveCursor(Direction::Down) => {
                cursor.1 = wrap_incr(cursor.1, Layout::GRID_SIZE)
            }
            Command::MoveCursor(Direction::Left) => {
                cursor.0 = wrap_decr(cursor.0, Layout::GRID_SIZE)
            }
            Command::MoveCursor(Direction::Right) => {
                cursor.0 = wrap_incr(cursor.0, Layout::GRID_SIZE)
            }
            Command::MoveCursor(Direction::Up) => cursor.1 = wrap_decr(cursor.1, Layout::GRID_SIZE),
            Command::Cancel => game_events.push_back(GameEvent::CardDeselected),
            Command::Confirm => {
                let position = Position::Board(cursor.0, cursor.1);
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
        let Some(Position::Hand(selected_hand_idx)) = components.position[entity.index()] else {
            return;
        };

        components.position[entity.index()] = Some(position);
        let player = &components.owner[entity.index()];

        for e in 0..components.owner.len() {
            if &components.owner[e] != player {
                continue;
            }

            let Some(Position::Hand(k)) = components.position[e].as_mut() else {
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
    mstate: &MatchState,
    components: &Components,
    card_db: &CardDb,
) {
    let MatchState::Turn {
        phase: TurnPhase::ResolveRules { entity },
        ..
    } = mstate
    else {
        return;
    };

    let Some(placed_card) = get_card_view(*entity, components, card_db) else {
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
        if let Some(player) = owners[entity.index()].as_mut() {
            *player = !*player;
            events_out.push_back(GameEvent::CardFlipped);
        }
    }
}

pub fn win_system(
    events_out: &mut VecDeque<GameEvent>,
    mstate: MatchState,
    components: &Components,
) {
    let MatchState::Turn {
        phase: TurnPhase::End,
        ..
    } = mstate
    else {
        return;
    };

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
    mstate: &MatchState,
    components: &Components,
    card_db: &CardDb,
) -> Result<(), String> {
    let Theme { bg, fg } = ctx.ui.palette.mono;

    ctx.canvas.set_draw_color(bg);
    ctx.canvas.clear();

    render_board(ctx)?;

    // render cards
    let active_entity = match mstate {
        MatchState::Turn {
            phase: TurnPhase::SelectCard { entity, .. },
            ..
        } => entity,

        MatchState::Turn {
            phase: TurnPhase::PlaceCard { entity, .. },
            ..
        } => &Some(*entity),

        MatchState::Turn {
            phase: TurnPhase::ResolveRules { entity, .. },
            ..
        } => &Some(*entity),

        _ => &None,
    };
    for entity in Entity::iter() {
        render_card(ctx, entity, *active_entity, components, card_db)?;
    }

    // render cursor
    match mstate {
        MatchState::Turn {
            phase: TurnPhase::SelectCard { cursor, .. },
            player: Player::P1,
        } => {
            let s_cursor = ctx.asset_manager.get_sprite("cursor").unwrap();
            let t_cursor = ctx
                .asset_manager
                .get_texture_mut(s_cursor.texture_id)
                .unwrap();
            t_cursor.set_color_mod(fg.r, fg.g, fg.b);

            let card_rect = ctx.ui.layout.hand.p1[*cursor];
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

        MatchState::Turn {
            phase: TurnPhase::SelectCard { cursor, .. },
            player: Player::P2,
        } => {
            let s_cursor = ctx.asset_manager.get_sprite("cursor").unwrap();
            let t_cursor = ctx
                .asset_manager
                .get_texture_mut(s_cursor.texture_id)
                .unwrap();
            t_cursor.set_color_mod(fg.r, fg.g, fg.b);

            let card_rect = ctx.ui.layout.hand.p2[*cursor];
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

        MatchState::Turn {
            phase: TurnPhase::PlaceCard { cursor, .. },
            ..
        } => {
            let j = cursor.1 * 3 + cursor.0; // FIXME magic number
            let card_rect = ctx.ui.layout.board[j];
            let mut cursor = card_rect.left_shifted(8).top_shifted(8);
            cursor.resize(card_rect.width() + 16, card_rect.height() + 16);

            ctx.canvas.set_draw_color(fg);
            ctx.canvas.draw_rect(cursor)?;
        }

        _ => {}
    }

    ctx.canvas.present();

    Ok(())
}

/// Returns whether the game is running or not.
pub fn director_system(
    events: &VecDeque<GameEvent>,
    mstate: &mut MatchState,
    position: &[Option<Position>],
) {
    *mstate = match mstate {
        MatchState::GameStart => MatchState::Turn {
            phase: TurnPhase::Start,
            player: Player::P1,
        },

        MatchState::Turn {
            phase: TurnPhase::Start,
            player,
        } => MatchState::Turn {
            phase: TurnPhase::SelectCard {
                cursor: 0,
                entity: None,
            },
            player: *player,
        },

        MatchState::Turn {
            phase: TurnPhase::SelectCard { .. },
            player,
        } => {
            if let Some(GameEvent::CardSelected { target }) = events
                .iter()
                .find(|e| matches!(e, GameEvent::CardSelected { .. }))
            {
                MatchState::Turn {
                    phase: TurnPhase::PlaceCard {
                        cursor: (1, 1),
                        entity: *target,
                    },
                    player: *player,
                }
            } else {
                *mstate
            }
        }

        MatchState::Turn {
            phase: TurnPhase::PlaceCard { entity, .. },
            player,
        } => {
            let deselected = events
                .iter()
                .any(|e| matches!(e, GameEvent::CardDeselected));
            let placed = events.iter().any(|e| matches!(e, GameEvent::CardPlaced));

            if deselected {
                let cursor = position[entity.index()].map_or(0, |pos| match pos {
                    Position::Hand(j) => j,
                    _ => 0,
                });

                MatchState::Turn {
                    phase: TurnPhase::SelectCard {
                        cursor,
                        entity: None,
                    },
                    player: *player,
                }
            } else if placed {
                MatchState::Turn {
                    phase: TurnPhase::ResolveRules { entity: *entity },
                    player: *player,
                }
            } else {
                *mstate
            }
        }

        MatchState::Turn {
            phase: TurnPhase::ResolveRules { .. },
            player,
        } => MatchState::Turn {
            phase: TurnPhase::End,
            player: *player,
        },

        MatchState::Turn {
            phase: TurnPhase::End,
            player,
        } => {
            let match_ended = events.iter().any(|e| matches!(e, GameEvent::MatchEnded(_)));

            if match_ended {
                MatchState::GameOver
            } else {
                MatchState::Turn {
                    phase: TurnPhase::Start,
                    player: !*player,
                }
            }
        }

        MatchState::GameOver => *mstate,
    };
}
