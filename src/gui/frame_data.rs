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
            let clr = if frame.cancelled {
                particle_hit_color()
            } else {
                frame_startup_color()
            };

            ctx.set(x_start + x, y_start, clr, bg_color(), 219);
        }

        for x in 0..frame.active {
            let clr = if frame.cancelled {
                particle_hit_color()
            } else {
                frame_active_color()
            };

            ctx.set(x_start + x + frame.startup, y_start, clr, bg_color(), 219);
        }

        for x in 0..frame.recovery {
            let clr = if frame.cancelled {
                particle_hit_color()
            } else {
                frame_recovery_color()
            };

            ctx.set(
                x_start + x + frame.startup + frame.active,
                y_start,
                clr,
                bg_color(),
                219,
            );
        }

        ctx.set_active_console(1);
        ctx.set(
            x_start + frame.current,
            y_start,
            frame_current_color(),
            bg_color(),
            223,
        );

        if frame.cancelled {
            if frame.current < frame.startup + frame.active {
                ctx.print(x_start, y_start + 1, "Interrupted!");
            } else {
                ctx.print(x_start, y_start + 1, "Punished!");
            }
        }
    }
}
