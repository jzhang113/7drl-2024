use super::consts::*;
use crate::*;
use rltk::{Algorithm2D};

pub fn draw_all(ecs: &World, ctx: &mut Rltk, is_weapon_sheathed: bool) {
    // map elements
    draw_map(ecs, ctx);
    draw_renderables(ecs, ctx, is_weapon_sheathed);
    // draw_blocked_tiles(ecs, ctx);
    draw_attacks_in_progress(ecs, ctx);
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        MAP_SCREEN_X - 1,
        MAP_SCREEN_Y - 1,
        camera::VIEW_W + 1,
        camera::VIEW_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let map = ecs.fetch::<Map>();
    let floor_str = map.name.clone();
    ctx.print(
        camera::VIEW_W + MAP_SCREEN_X - floor_str.len() as i32,
        MAP_SCREEN_Y - 1,
        floor_str,
    );

    for idx in map.camera.iter() {
        if map.known_tiles[idx] || SHOW_MAP {
            let (symbol, mut fg) = match map.tiles[idx] {
                TileType::Floor => (rltk::to_cp437('.'), map_floor_color()),
                TileType::Wall => (rltk::to_cp437('#'), map.color_map[idx]),
                TileType::DownStairs => (rltk::to_cp437('>'), map_exit_color()),
                TileType::NewLevel => (rltk::to_cp437('>'), map_exit_color()),
            };

            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }

            set_map_tile(
                ctx,
                &map.camera.origin,
                &map.index_to_point2d(idx),
                fg,
                symbol,
            );
        }
    }
}

pub fn draw_renderables(ecs: &World, ctx: &mut Rltk, is_weapon_sheathed: bool) {
    let entities = ecs.entities();
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let particles = ecs.read_storage::<ParticleLifetime>();
    let multitiles = ecs.read_storage::<MultiTile>();
    let facings = ecs.read_storage::<Facing>();
    let map = ecs.fetch::<Map>();
    let player = ecs.fetch::<Entity>();

    for (pos, render, particle) in (&positions, &renderables, &particles).join() {
        if !map.camera.on_screen(pos.as_point()) {
            continue;
        }

        let mut fg = render.fg;
        let mut bg = render.bg;

        if particle.should_fade {
            let fade_percent = ezing::expo_inout(1.0 - particle.remaining / particle.base);
            let base_color = bg_color();

            fg = fg.lerp(base_color, fade_percent);
            bg = bg.lerp(base_color, fade_percent);
        }

        if map.visible_tiles[map.get_index(pos.x, pos.y)] || SHOW_REND {
            ctx.set_active_console(0);
            set_map_tile_with_bg(
                ctx,
                &map.camera.origin,
                &pos.as_point(),
                fg,
                bg,
                render.symbol,
            );
            ctx.set_active_console(1);
        }
    }

    for (ent, pos, render, mtt, facing) in (
        &entities,
        &positions,
        &renderables,
        (&multitiles).maybe(),
        (&facings).maybe(),
    )
        .join()
    {
        let symbol = if let Some(facing) = facing {
            if ent != *player || !is_weapon_sheathed {
                match facing.direction {
                    Direction::N => rltk::to_cp437('^'),
                    Direction::E => rltk::to_cp437('>'),
                    Direction::S => rltk::to_cp437('v'),
                    Direction::W => rltk::to_cp437('<'),
                }
            } else {
                render.symbol
            }
        } else {
            render.symbol
        };

        if map.visible_tiles[map.get_index(pos.x, pos.y)] || SHOW_REND {
            set_map_tile_with_bg(
                ctx,
                &map.camera.origin,
                &pos.as_point(),
                render.fg,
                render.bg,
                symbol,
            );
        }

        if let Some(mtt) = mtt {
            for part_list in &mtt.part_list {
                for (mtt_pos, mtt_symbol) in &part_list.symbol_map {
                    if map.visible_tiles[map.get_index(pos.x + mtt_pos.x, pos.y + mtt_pos.y)]
                        || SHOW_REND
                    {
                        set_map_tile_with_bg(
                            ctx,
                            &map.camera.origin,
                            &(pos.as_point() + *mtt_pos),
                            render.fg,
                            render.bg,
                            *mtt_symbol,
                        );
                    }
                }
            }
        }
    }
}

pub fn draw_blocked_tiles(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    ctx.set_active_console(0);
    for index in map.camera.iter() {
        if map.blocked_tiles[index] {
            let point = map.index_to_point2d(index);
            highlight_bg(
                ctx,
                &map.camera.origin,
                &point,
                RGB::named(rltk::DARKSLATEGRAY),
            );
        }
    }
    ctx.set_active_console(1);
}

pub fn draw_attacks_in_progress(ecs: &World, ctx: &mut Rltk) {
    let attacks = ecs.read_storage::<AttackIntent>();
    let in_progress = ecs.read_storage::<AttackInProgress>();
    let map = ecs.fetch::<Map>();

    for (attack, _) in (&attacks, &in_progress).join() {
        ctx.set_active_console(0);
        for point in attack_type::each_attack_target(attack.main, attack.loc) {
            if !map.camera.on_screen(point) {
                continue;
            }

            highlight_bg(ctx, &map.camera.origin, &point, RGB::named(rltk::DARKRED));
        }
        ctx.set_active_console(1);
    }
}

fn set_map_tile(
    ctx: &mut Rltk,
    camera_pos: &rltk::Point,
    pos: &rltk::Point,
    fg: RGB,
    symbol: rltk::FontCharType,
) {
    ctx.set(
        MAP_SCREEN_X + pos.x - camera_pos.x,
        MAP_SCREEN_Y + pos.y - camera_pos.y,
        fg,
        bg_color(),
        symbol,
    );
}

fn set_map_tile_with_bg(
    ctx: &mut Rltk,
    camera_pos: &rltk::Point,
    pos: &rltk::Point,
    fg: RGB,
    bg: RGB,
    symbol: rltk::FontCharType,
) {
    ctx.set(
        MAP_SCREEN_X + pos.x - camera_pos.x,
        MAP_SCREEN_Y + pos.y - camera_pos.y,
        fg,
        bg,
        symbol,
    );
}

fn highlight_bg(ctx: &mut Rltk, camera_pos: &rltk::Point, pos: &rltk::Point, color: RGB) {
    ctx.set_bg(
        MAP_SCREEN_X + pos.x - camera_pos.x,
        MAP_SCREEN_Y + pos.y - camera_pos.y,
        color,
    );
}

pub fn draw_viewable_info(ecs: &World, ctx: &mut Rltk, entity: &Entity, index: u32) {
    let selected_color = select_highlight_color();
    let bg_color = bg_color();

    ctx.set(
        0,
        2 * index + 1,
        text_highlight_color(),
        bg_color,
        rltk::to_cp437('>'),
    );

    let positions = ecs.read_storage::<Position>();
    let viewables = ecs.read_storage::<Viewable>();
    let healths = ecs.read_storage::<Health>();
    let atk_in_progress = ecs.read_storage::<AttackInProgress>();
    let blocking = ecs.read_storage::<BlockAttack>();
    let map = ecs.fetch::<Map>();

    let pos = positions
        .get(*entity)
        .expect("viewable didn't have a position");
    let view = viewables.get(*entity).expect("viewable didn't have a view");
    let health = healths.get(*entity).expect("viewable didn't have health");

    let x = MAP_SCREEN_X + pos.x - map.camera.origin.x;
    let y = MAP_SCREEN_Y + pos.y - map.camera.origin.y;

    highlight_bg(
        ctx,
        &map.camera.origin,
        &Position::as_point(pos),
        selected_color,
    );

    let (box_x, box_y) = position_box(ctx, x, y, 15, 10, selected_color, bg_color);

    ctx.print(box_x + 1, box_y, view.name.clone());
    ctx.print(
        box_x + 1,
        box_y + 1,
        format!("HP: {}/{}", health.current, health.max),
    );

    if atk_in_progress.get(*entity).is_some() {
        ctx.print(box_x + 1, box_y + 3, "Attacking");
    } else if blocking.get(*entity).is_some() {
        ctx.print(box_x + 1, box_y + 3, "Blocking");
    } else {
        ctx.print(box_x + 1, box_y + 3, "Idle");
    }

    for (i, line) in view.description.iter().enumerate() {
        ctx.print(box_x + 1, box_y + 5 + i as i32, line.clone());
    }
}

// draw a box stemming from a given point
// returns the top left of the new box
fn position_box(ctx: &mut Rltk, x: i32, y: i32, w: i32, h: i32, fg: RGB, bg: RGB) -> (i32, i32) {
    let right = x + w < CONSOLE_WIDTH - 1;
    let down = y + h < camera::VIEW_H;

    // boxes prefer to be right and down if several positions are possible
    if right {
        if down {
            ctx.draw_box(x + 1, y, w, h, fg, bg);
            ctx.set(x + 1, y, fg, bg, rltk::to_cp437('┬'));
            return (x + 1, y);
        } else {
            ctx.draw_box(x + 1, y - h, w, h, fg, bg);
            ctx.set(x + 1, y, fg, bg, rltk::to_cp437('┴'));
            return (x + 1, y - h);
        }
    } else {
        if down {
            ctx.draw_box(x - w - 1, y, w, h, fg, bg);
            ctx.set(x - 1, y, fg, bg, rltk::to_cp437('┬'));
            return (x - w - 1, y);
        } else {
            ctx.draw_box(x - w - 1, y - h, w, h, fg, bg);
            ctx.set(x - 1, y, fg, bg, rltk::to_cp437('┴'));
            return (x - w - 1, y - h);
        }
    }
}
