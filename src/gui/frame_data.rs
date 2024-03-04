use super::consts::*;
use crate::*;

pub fn draw_frames(gs: &State, ctx: &mut Rltk) {
    let frames = gs.ecs.read_storage::<FrameData>();
    let players = gs.ecs.read_storage::<Player>();

    let x_start = (SIDE_X + SIDE_W + 1) as u32;
    let y_start = SIDE_H as u32;

    ctx.print(x_start, y_start - 1, gs.tick);

    for (_, frame) in (&players, &frames).join() {
        ctx.set_active_console(0);
        for x in 0..frame.startup {
            ctx.set(
                x_start + x,
                y_start,
                RGB::named(rltk::GREEN),
                RGB::named(rltk::BLACK),
                219,
            );
        }

        for x in 0..frame.active {
            ctx.set(
                x_start + x + frame.startup,
                y_start,
                RGB::named(rltk::ORANGE),
                RGB::named(rltk::BLACK),
                219,
            );
        }

        for x in 0..frame.recovery {
            ctx.set(
                x_start + x + frame.startup + frame.active,
                y_start,
                RGB::named(rltk::BLUE),
                RGB::named(rltk::BLACK),
                219,
            );
        }

        ctx.set_active_console(1);
        ctx.set(
            x_start + frame.current,
            y_start,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
            223,
        );
    }
}
