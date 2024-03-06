use crate::*;
use rltk::{Point, VirtualKeyCode};

pub const DODGE_STAM_REQ: i32 = 3;
pub const CHARGE_STAM_REQ: i32 = 2;

fn try_move_player(ecs: &mut World, dx: i32, dy: i32) -> RunState {
    use std::cmp::{max, min};
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut movements = ecs.write_storage::<MoveIntent>();
    let mut attacks = ecs.write_storage::<AttackIntent>();
    let mut frames = ecs.write_storage::<FrameData>();
    let mut healths = ecs.write_storage::<Health>();
    let openables = ecs.read_storage::<Openable>();
    let npcs = ecs.read_storage::<Npc>();
    let map = ecs.fetch::<Map>();
    let player = ecs.fetch::<Entity>();
    let mut log = ecs.fetch_mut::<gamelog::GameLog>();

    for (_player, pos) in (&players, &mut positions).join() {
        let new_x = min(map.width, max(0, pos.x + dx));
        let new_y = min(map.height, max(0, pos.y + dy));
        let dest_index = map.get_index(new_x, new_y);

        match map.tiles[dest_index] {
            TileType::DownStairs => return RunState::ChangeMap,
            TileType::NewLevel => return RunState::GenerateLevel,
            _ => {}
        }

        if !map.blocked_tiles[dest_index] {
            let new_move = MoveIntent {
                loc: Point::new(new_x, new_y),
                force_facing: None,
                delay: 0,
            };
            movements
                .insert(*player, new_move)
                .expect("Failed to insert new movement from player");

            return RunState::Running;
        } else if map.tiles[dest_index] != crate::TileType::Wall {
            if let Some(dest_ent) = map.creature_map.get(&dest_index) {
                if let Some(_) = openables.get(*dest_ent) {
                    if let Some(health) = healths.get_mut(*dest_ent) {
                        // will be cleaned up by sys_death
                        health.current = 0;
                    }

                    return RunState::Running;
                } else if let Some(npc) = npcs.get(*dest_ent) {
                    match npc.npc_type {
                        NpcType::Blacksmith => {
                            log.add("Upgrade your equipment here");
                            return RunState::Blacksmith;
                        }
                        NpcType::Handler => {
                            log.add("Accept missions here");
                            return RunState::MissionSelect { index: 0 };
                        }
                        NpcType::Shopkeeper => {
                            log.add("Buy useful items here");
                            return RunState::Shop;
                        }
                    }
                } else {
                    // bump attack
                    let attack = crate::attack_type::get_attack_intent(
                        AttackType::Melee,
                        Point::new(new_x, new_y),
                        None,
                    );
                    let frame = crate::attack_type::get_frame_data(AttackType::Melee);

                    attacks
                        .insert(*player, attack)
                        .expect("Failed to insert new attack from player");

                    frames.insert(*player, frame).ok();

                    return RunState::Running;
                    // Keep bump attacks?
                    // return RunState::AwaitingInput;
                }
            }

            return RunState::AwaitingInput;
        }
    }

    RunState::AwaitingInput
}

fn try_move_charging(
    gs: &mut State,
    input_dir: crate::Direction,
    movement_dir: crate::Direction,
) -> RunState {
    if input_dir == movement_dir {
        return RunState::Running;
    } else if input_dir == movement_dir.opp() {
        gs.player_charging.1 = input_dir;
        gs.player_charging.2 = 1;
        return RunState::Running;
    } else if input_dir == movement_dir.left() || input_dir == movement_dir.right() {
        let dir_point = input_dir.to_point();
        return try_move_player(&mut gs.ecs, dir_point.x, dir_point.y);
    }

    RunState::Running
}

fn handle_attack(gs: &mut State, data: AttackData) -> RunState {
    let mut attacks = gs.ecs.write_storage::<AttackIntent>();
    let mut frames = gs.ecs.write_storage::<FrameData>();
    let positions = gs.ecs.read_storage::<Position>();
    let facings = gs.ecs.read_storage::<Facing>();
    let mut stams = gs.ecs.write_storage::<Stamina>();
    let player = gs.ecs.fetch::<Entity>();

    let pos = positions.get(*player).unwrap();
    let facing = facings.get(*player).unwrap();
    let stamina = stams.get_mut(*player).unwrap();

    if data.stam_cost > stamina.current {
        return RunState::AwaitingInput;
    }

    if data.needs_target {
        return RunState::Targetting {
            attack_type: data.attack_type,
            cursor_point: pos.as_point(),
            validity_mode: TargettingValid::All,
            show_path: data.needs_path,
        };
    }

    stamina.current -= data.stam_cost;
    stamina.recover = false;

    let intent = AttackIntent {
        main: data.attack_type,
        loc: pos.as_point(),
    };
    attacks
        .insert(*player, intent)
        .expect("Failed to insert new attack from player");
    frames.insert(*player, data.frame_data).ok();

    RunState::Running
}

fn handle_charging(gs: &mut State) -> bool {
    let player = gs.ecs.fetch::<Entity>();
    let map = gs.ecs.fetch::<Map>();

    let mut movements = gs.ecs.write_storage::<MoveIntent>();
    let mut attacks = gs.ecs.write_storage::<AttackIntent>();
    let mut frames = gs.ecs.write_storage::<FrameData>();

    let (mut player_x, mut player_y) = {
        let pos = gs.ecs.read_storage::<Position>();
        let p = pos.get(*player).unwrap();
        (p.x, p.y)
    };

    for _ in 0..gs.player_charging.2 {
        let curr_point = rltk::Point::new(player_x, player_y);
        let next_point =
            crate::direction::Direction::point_in_direction(curr_point, gs.player_charging.1);

        let dest_index = map.get_index(next_point.x, next_point.y);
        if !map.blocked_tiles[dest_index] {
            player_x = next_point.x;
            player_y = next_point.y;
            continue;
        }

        // If we hit an obstacle, move to the last legal position and stop
        let new_move = MoveIntent {
            loc: curr_point,
            force_facing: None,
            delay: 0,
        };
        movements
            .insert(*player, new_move)
            .expect("Failed to insert new movement from player");
        gs.player_charging.0 = false;

        // If the obstacle happens to be a creature, also put in an attack (bump?)
        if let Some(_dest_ent) = map.creature_map.get(&dest_index) {
            let attack = AttackIntent {
                main: AttackType::Melee,
                loc: next_point,
            };

            attacks
                .insert(*player, attack)
                .expect("Failed to insert new attack from player");
            frames
                .insert(*player, get_frame_data(AttackType::Melee))
                .ok();

            return false;
        }

        return false;
    }

    let new_move = MoveIntent {
        loc: rltk::Point::new(player_x, player_y),
        force_facing: None,
        delay: 0,
    };
    movements
        .insert(*player, new_move)
        .expect("Failed to insert new movement from player");

    // If we did not stop charging, increase speed if possible
    if gs.player_charging.2 < 4 {
        gs.player_charging.2 += 1;
    }

    true
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    {
        let can_act = gs.ecs.read_storage::<super::CanActFlag>();
        let player = gs.ecs.fetch::<Entity>();
        can_act
            .get(*player)
            .expect("player_input called, but it is not your turn");
    };

    if gs.player_charging.0 {
        // check bool that auto-movement only happens once
        if !gs.player_charging.3 {
            let can_player_take_action = handle_charging(gs);

            if !can_player_take_action {
                return RunState::Running;
            } else {
                // process the movement once now before handling player input
                sys_movement::MovementSystem.run_now(&gs.ecs);
                gs.player_charging.3 = true;
            }
        }

        let next_state = match ctx.key {
            None => RunState::AwaitingInput,
            Some(key) => match key {
                VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                    try_move_charging(gs, crate::Direction::W, gs.player_charging.1)
                }
                VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                    try_move_charging(gs, crate::Direction::E, gs.player_charging.1)
                }
                VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                    try_move_charging(gs, crate::Direction::N, gs.player_charging.1)
                }
                VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                    try_move_charging(gs, crate::Direction::S, gs.player_charging.1)
                }
                VirtualKeyCode::Period => RunState::Running,
                _ => RunState::AwaitingInput,
            },
        };

        if next_state == RunState::Running {
            gs.player_charging.3 = false;

            // end charging if we run out of stamina
            let mut stams = gs.ecs.write_storage::<Stamina>();
            let player = gs.ecs.fetch::<Entity>();
            let stamina = stams.get_mut(*player).unwrap();

            if stamina.current < CHARGE_STAM_REQ {
                gs.player_charging.0 = false;
                return RunState::Running;
            } else {
                stamina.current -= CHARGE_STAM_REQ;
                stamina.recover = false;
            }
        }

        next_state
    } else {
        handle_keys(gs, ctx)
    }
}

fn handle_dodge(ecs: &mut World) -> Option<MoveIntent> {
    let player = ecs.fetch::<Entity>();
    let map = ecs.fetch::<Map>();
    let facing = ecs.read_storage::<Facing>();

    let (mut player_x, mut player_y, player_facing) = {
        let pos = ecs.read_storage::<Position>();
        let p = pos.get(*player).unwrap();
        let f = facing.get(*player).unwrap();
        (p.x, p.y, f.direction)
    };

    let (initial_x, initial_y) = (player_x, player_y);

    let backhop_dir = match player_facing {
        Direction::N => Direction::S,
        Direction::E => Direction::W,
        Direction::S => Direction::N,
        Direction::W => Direction::E,
    };

    for _ in 0..2 {
        let curr_point = rltk::Point::new(player_x, player_y);
        let next_point = crate::direction::Direction::point_in_direction(curr_point, backhop_dir);
        let dest_index = map.get_index(next_point.x, next_point.y);

        if map.blocked_tiles[dest_index] {
            break;
        } else {
            player_x = next_point.x;
            player_y = next_point.y;
        }
    }

    if initial_x == player_x && initial_y == player_y {
        return None;
    }

    Some(MoveIntent {
        loc: rltk::Point::new(player_x, player_y),
        force_facing: Some(player_facing),
        delay: 0,
    })
}

pub fn can_dodge(gs: &State) -> bool {
    let stams = gs.ecs.read_storage::<Stamina>();
    let player = gs.ecs.fetch::<Entity>();
    let stamina = stams.get(*player).unwrap();
    stamina.current >= DODGE_STAM_REQ
}

fn reduce_stam_for_dodge(ecs: &mut World) {
    let mut stams = ecs.write_storage::<Stamina>();
    let player = ecs.fetch::<Entity>();
    let stamina = stams.get_mut(*player).unwrap();
    stamina.current -= DODGE_STAM_REQ;
    stamina.recover = false;
}

pub fn end_turn_cleanup(ecs: &mut World) {
    // remove can act flag
    // let player = ecs.fetch::<Entity>();
    let mut can_act = ecs.write_storage::<super::CanActFlag>();
    // let mut can_react = ecs.write_storage::<super::CanReactFlag>();

    // let is_reaction = {
    //     let can_act = ecs.read_storage::<super::CanActFlag>();
    //     let player = ecs.fetch::<Entity>();
    //     can_act
    //         .get(*player)
    //         .expect("player_input called, but it is not your turn")
    //         .is_reaction
    // };

    // if is_reaction {
    //     can_react.remove(*player);
    // } else {
    //     can_react
    //         .insert(*player, super::CanReactFlag {})
    //         .expect("Failed to insert CanReactFlag");
    // }

    can_act.clear();

    // clear message line
    let mut log = ecs.fetch_mut::<GameLog>();
    log.dirty = false;
}

fn handle_keys(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                let next_state = try_move_player(&mut gs.ecs, -1, 0);
                next_state
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                let next_state = try_move_player(&mut gs.ecs, 1, 0);
                next_state
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                let next_state = try_move_player(&mut gs.ecs, 0, -1);
                next_state
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                let next_state = try_move_player(&mut gs.ecs, 0, 1);
                next_state
            }
            VirtualKeyCode::Period => RunState::Running,
            // VirtualKeyCode::D => {
            //     // TODO: For testing, remove
            //     return RunState::Dead { success: true };
            // }
            // VirtualKeyCode::V => RunState::ViewEnemy { index: 0 },
            VirtualKeyCode::Space => {
                if !can_dodge(gs) {
                    RunState::AwaitingInput
                } else {
                    let p = {
                        let player = gs.ecs.fetch::<Entity>();
                        let pos = gs.ecs.read_storage::<Position>();
                        pos.get(*player).unwrap().as_point()
                    };

                    return RunState::Targetting {
                        attack_type: AttackType::Dodge,
                        cursor_point: p,
                        validity_mode: crate::TargettingValid::Unblocked,
                        show_path: true,
                    };
                }
            }
            VirtualKeyCode::A => RunState::AbilitySelect { index: 0 },
            _ => RunState::AwaitingInput,
        },
    }
}

fn apply_invuln(ecs: &mut World) {
    let mut invulns = ecs.write_storage::<Invulnerable>();
    let player = ecs.fetch::<Entity>();

    invulns
        .insert(*player, Invulnerable { duration: 6 }) // 24 / 4 = 6 ticks
        .expect("Failed to make player invulnerable");
}

pub enum SelectionResult {
    Selected,
    Canceled,
    NoResponse,
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    cursor: rltk::Point,
    tiles_in_range: Vec<Point>,
    validity_mode: crate::TargettingValid,
    show_path: bool,
) -> (SelectionResult, Option<Point>) {
    let player = gs.ecs.fetch::<Entity>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    let map = gs.ecs.fetch::<Map>();
    let camera_point = map.camera.origin;
    let positions = gs.ecs.read_storage::<Position>();

    let mut valid_target = false;

    if validity_mode == TargettingValid::None {
        ctx.print_color(
            gui::consts::MAP_SCREEN_X,
            gui::consts::MAP_SCREEN_Y - 1,
            crate::header_message_color(),
            crate::bg_color(),
            "Confirm use",
        );
    } else {
        ctx.set_active_console(0);

        // Highlight available target cells
        let mut available_cells = Vec::new();

        if let Some(viewshed) = viewsheds.get(*player) {
            // We have a viewshed
            for point in &viewshed.visible {
                if tiles_in_range.contains(point) {
                    ctx.set_bg(
                        gui::consts::MAP_SCREEN_X + point.x - camera_point.x,
                        gui::consts::MAP_SCREEN_Y + point.y - camera_point.y,
                        crate::tiles_in_range_color(),
                    );
                    available_cells.push(point);
                }
            }
        }

        // Apply validity
        let valid_cells = match validity_mode {
            TargettingValid::Unblocked => available_cells
                .iter()
                .filter(|point| {
                    let idx = map.get_index(point.x, point.y);
                    !map.blocked_tiles[idx]
                })
                .copied()
                .collect(),
            TargettingValid::Occupied => available_cells
                .iter()
                .filter(|point| {
                    let idx = map.get_index(point.x, point.y);
                    map.creature_map.contains_key(&idx)
                })
                .copied()
                .collect(),
            TargettingValid::None => Vec::new(),
            TargettingValid::All => available_cells,
        };

        // Draw cursor
        valid_target = valid_cells
            .iter()
            .any(|pos| pos.x == cursor.x && pos.y == cursor.y);

        let cursor_color = if valid_target {
            crate::valid_cursor_color()
        } else {
            crate::invalid_cursor_color()
        };

        ctx.set_bg(
            gui::consts::MAP_SCREEN_X + cursor.x - camera_point.x,
            gui::consts::MAP_SCREEN_Y + cursor.y - camera_point.y,
            cursor_color,
        );

        if show_path {
            let player_point = positions.get(*player).unwrap().as_point();
            let mut path = rltk::line2d_bresenham(cursor, player_point);
            path.pop();

            for path_point in path {
                ctx.set_bg(
                    gui::consts::MAP_SCREEN_X + path_point.x - camera_point.x,
                    gui::consts::MAP_SCREEN_Y + path_point.y - camera_point.y,
                    cursor_color,
                )
            }
        }

        ctx.set_active_console(1);

        if valid_target {
            ctx.print_color(
                crate::gui::consts::MAP_SCREEN_X,
                crate::gui::consts::MAP_SCREEN_Y - 1,
                crate::header_message_color(),
                crate::bg_color(),
                "Select Target",
            );
        } else {
            ctx.print_color(
                gui::consts::MAP_SCREEN_X,
                gui::consts::MAP_SCREEN_Y - 1,
                crate::header_err_color(),
                crate::bg_color(),
                "Invalid Target",
            );
        }
    }

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Escape => return (SelectionResult::Canceled, None),
            VirtualKeyCode::Space | VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                if valid_target {
                    return (
                        SelectionResult::Selected,
                        Some(Point::new(cursor.x, cursor.y)),
                    );
                }

                if validity_mode == TargettingValid::None {
                    return (SelectionResult::Selected, None);
                }

                return (SelectionResult::Canceled, None);
            }
            VirtualKeyCode::Tab => {
                let length = gs.tab_targets.len();

                if length > 0 {
                    gs.tab_index += 1;
                    return (
                        SelectionResult::NoResponse,
                        Some(gs.tab_targets[gs.tab_index % length]),
                    );
                }
            }
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                return (
                    SelectionResult::NoResponse,
                    Some(rltk::Point::new(cursor.x - 1, cursor.y)),
                );
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                return (
                    SelectionResult::NoResponse,
                    Some(rltk::Point::new(cursor.x + 1, cursor.y)),
                );
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                return (
                    SelectionResult::NoResponse,
                    Some(rltk::Point::new(cursor.x, cursor.y - 1)),
                );
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                return (
                    SelectionResult::NoResponse,
                    Some(rltk::Point::new(cursor.x, cursor.y + 1)),
                );
            }
            _ => {}
        },
    };

    (SelectionResult::NoResponse, None)
}

pub fn view_input(gs: &mut State, ctx: &mut Rltk, index: u32) -> RunState {
    let entities = gs.ecs.entities();
    let v_indexes = gs.ecs.read_storage::<ViewableIndex>();
    let viewables = gs.ecs.read_storage::<Viewable>();

    let mut new_index = index;
    let mut max_index = 0;

    for (ent, viewables, v_index) in (&entities, &viewables, &v_indexes).join() {
        if let Some(list_index) = v_index.list_index {
            max_index = std::cmp::max(list_index, max_index);

            if list_index == index {
                gui::map::draw_viewable_info(&gs.ecs, ctx, &ent, index);
            }
        }
    }

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Escape => return RunState::AwaitingInput,
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if new_index > 0 {
                    new_index -= 1;
                } else {
                    new_index += max_index;
                }
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                new_index += 1;
            }
            _ => {}
        },
    }

    RunState::ViewEnemy {
        index: new_index % (max_index + 1),
    }
}

pub fn mission_select_input(gs: &mut State, ctx: &mut Rltk, index: usize) -> RunState {
    let mut new_index = index;
    let max_index = gs.quests.entries.len();

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if new_index > 0 {
                    new_index -= 1;
                } else {
                    new_index += max_index;
                }
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                new_index += 1;
            }
            VirtualKeyCode::Escape => {
                return RunState::Running;
            }
            VirtualKeyCode::Space | VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                gs.selected_quest = Some(gs.quests.entries[index].clone());
                return RunState::Running;
            }
            VirtualKeyCode::A => {
                gs.selected_quest = None;
            }
            _ => {}
        },
    }

    RunState::MissionSelect {
        index: new_index % max_index,
    }
}

pub fn ability_select_input(gs: &mut State, ctx: &mut Rltk, index: usize) -> RunState {
    let mut new_index = index;
    let max_index = gs.player_abilities.len();

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
                if new_index > 0 {
                    new_index -= 1;
                } else {
                    new_index += max_index;
                }
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
                new_index += 1;
            }
            VirtualKeyCode::Escape => {
                return RunState::AwaitingInput;
            }
            VirtualKeyCode::Space | VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                return handle_attack(gs, gs.player_abilities[index].clone())
            }
            _ => {
                let selection = rltk::letter_to_option(key) as usize;
                if selection < max_index {
                    return handle_attack(gs, gs.player_abilities[selection].clone());
                }
            }
        },
    }

    RunState::AbilitySelect {
        index: new_index % max_index,
    }
}
