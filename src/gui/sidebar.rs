use super::consts::*;
use crate::*;

pub fn draw_sidebar(gs: &State, ctx: &mut Rltk) {
    let players = gs.ecs.read_storage::<Player>();
    let healths = gs.ecs.read_storage::<Health>();
    let stams = gs.ecs.read_storage::<Stamina>();
    let views = gs.ecs.read_storage::<Viewable>();

    let rends = gs.ecs.read_storage::<Renderable>();
    let viewables = gs.ecs.read_storage::<Viewable>();
    let positions = gs.ecs.read_storage::<Position>();
    let movesets = gs.ecs.read_storage::<Moveset>();
    let frames = gs.ecs.read_storage::<FrameData>();
    let schedulables = gs.ecs.read_storage::<Schedulable>();

    let map = gs.ecs.fetch::<Map>();
    let player = gs.ecs.fetch::<Entity>();
    // let m_info = gs.ecs.fetch::<MissionInfo>();
    // let next_state = gs.ecs.fetch::<RunState>();

    let mouse_point = ctx.mouse_point();
    let adjusted_point = mouse_point - rltk::Point::new(SIDE_W + 1, 1) + map.camera.origin;

    ctx.draw_box(
        SIDE_X,
        SIDE_Y,
        SIDE_W,
        SIDE_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let mut x = SIDE_X + 1;
    let mut y = SIDE_Y + 1;

    for (_, rend, view, pos, stamina, health, frame) in (
        &players,
        &rends,
        &viewables,
        &positions,
        &stams,
        &healths,
        (&frames).maybe(),
    )
        .join()
    {
        ctx.set(x, y, rend.fg, rend.bg, rend.symbol);
        if adjusted_point.x == pos.x && adjusted_point.y == pos.y {
            ctx.print_color(
                x + 2,
                y,
                select_highlight_color(),
                bg_color(),
                view.name.clone(),
            );
        } else {
            ctx.print(x + 3, y, view.name.clone());
        }

        ctx.print(x, y + 1, "HP");
        draw_resource_bar(
            ctx,
            health.current,
            health.max,
            x + 3,
            y + 1,
            hp_main_color(),
            hp_alt_color(),
        );

        ctx.print(x, y + 2, "MP");
        draw_resource_bar(
            ctx,
            stamina.current,
            stamina.max,
            x + 3,
            y + 2,
            stam_main_color(),
            stam_alt_color(),
        );

        if let Some(frame) = frame {
            gui::frame_data::print_frame_state(ctx, frame, x as u32, y as u32 + 3);
        }
    }

    y += 2;
    for (rend, view, pos, health, frame, _) in (
        &rends,
        &viewables,
        &positions,
        (&healths).maybe(),
        (&frames).maybe(),
        &movesets,
    )
        .join()
    {
        if !map.visible_tiles[map.get_index(pos.x, pos.y)] {
            continue;
        }

        y += 3;
        ctx.set(x, y, rend.fg, rend.bg, rend.symbol);
        if adjusted_point.x == pos.x && adjusted_point.y == pos.y {
            ctx.print_color(
                x + 3,
                y,
                select_highlight_color(),
                bg_color(),
                view.name.clone(),
            );
        } else {
            ctx.print(x + 3, y, view.name.clone());
        }

        if let Some(health) = health {
            y += 1;
            ctx.print(x, y, "HP");
            draw_resource_bar(
                ctx,
                health.current,
                health.max,
                x + 3,
                y,
                hp_main_color(),
                hp_alt_color(),
            );
        }

        if let Some(frame) = frame {
            gui::frame_data::print_frame_state(ctx, frame, x as u32, y as u32 + 1);
        }
    }

    // Controls
    x = SIDE_W + 2;
    y = SIDE_H + 1;
    // ctx.print(x, y, "Controls");
    draw_movement_controls(ctx, x, y, text_highlight_color(), bg_color(), false);

    x += 11;
    ctx.print_color(x, y, text_highlight_color(), bg_color(), 'a');
    ctx.print(x + 1, y, "bility");

    x += 9;
    ctx.print_color(x, y, text_highlight_color(), bg_color(), 'i');
    ctx.print(x + 1, y, "nventory");

    // super::tooltip::draw_tooltips(&gs.ecs, ctx);
}

fn draw_resource_bar(
    ctx: &mut Rltk,
    curr: i32,
    max: i32,
    x: i32,
    y: i32,
    main_color: RGB,
    alt_color: RGB,
) {
    let curr = std::cmp::max(0, curr);
    for i in 0..curr {
        ctx.set(x + i, y, main_color, bg_color(), rltk::to_cp437('o'));
    }

    for i in curr..max {
        ctx.set(x + i, y, alt_color, bg_color(), rltk::to_cp437('o'));
    }
}

fn draw_movement_controls(ctx: &mut Rltk, x: i32, y: i32, fg: RGB, bg: RGB, inactive: bool) {
    ctx.set(x, y, fg, bg, 27);
    ctx.set(x + 1, y, fg, bg, 25);
    ctx.set(x + 2, y, fg, bg, 24);
    ctx.set(x + 3, y, fg, bg, 26);

    if inactive {
        ctx.print_color(x + 5, y, fg, bg, "move");
    } else {
        ctx.print(x + 5, y, "move");
    }
}
