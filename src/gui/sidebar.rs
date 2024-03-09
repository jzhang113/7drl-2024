use super::consts::*;
use crate::*;

pub fn draw_sidebar(gs: &State, ctx: &mut Rltk) {
    let players = gs.ecs.read_storage::<Player>();
    let healths = gs.ecs.read_storage::<Health>();
    let stams = gs.ecs.read_storage::<Stamina>();
    let views = gs.ecs.read_storage::<Viewable>();

    let player = gs.ecs.fetch::<Entity>();
    let m_info = gs.ecs.fetch::<MissionInfo>();
    let next_state = gs.ecs.fetch::<RunState>();

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

    for (_, stamina, health) in (&players, &stams, &healths).join() {
        draw_resource_bar(
            ctx,
            health.current,
            health.max,
            x,
            y,
            hp_main_color(),
            hp_alt_color(),
        );

        draw_resource_bar(
            ctx,
            stamina.current,
            stamina.max,
            x,
            y + 1,
            stam_main_color(),
            stam_alt_color(),
        );
    }

    let state = *gs.ecs.fetch::<RunState>();
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

    super::tooltip::draw_tooltips(&gs.ecs, ctx);
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
