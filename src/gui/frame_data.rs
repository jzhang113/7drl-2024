use super::consts::*;
use crate::*;

pub fn print_frame_state(ctx: &mut Rltk, frame: &FrameData, x: u32, y: u32) {
    ctx.set_active_console(1);

    if frame.cancelled {
        if frame.current < frame.startup + frame.active {
            ctx.print(x, y, "Interrupted!");
        } else {
            ctx.print(x, y, "Punished!");
        }
        return;
    }

    if frame.current <= frame.startup {
        ctx.print(x, y, "Startup");
        print_frame_progress(
            ctx,
            x,
            y,
            frame.current,
            frame.startup,
            frame_startup_color(),
        );
    } else if frame.current >= frame.startup + frame.active {
        ctx.print(x, y, "Recover");
        print_frame_progress(
            ctx,
            x,
            y,
            frame.current - frame.startup - frame.active,
            frame.recovery,
            frame_recovery_color(),
        );
    } else {
        ctx.print(x, y, "Active :");
        print_frame_progress(
            ctx,
            x,
            y,
            frame.current - frame.startup,
            frame.active,
            frame_active_color(),
        );
    }
}

fn print_frame_progress(ctx: &mut Rltk, x: u32, y: u32, current: u32, total: u32, color: RGB) {
    ctx.draw_bar_horizontal(x + 8, y, SIDE_W - 9, current, total, color, bg_color());

    ctx.set_active_console(0);
    let xt = if current < 10 { x + 9 } else { x + 8 };
    ctx.print(xt, y, current);
    ctx.print(x + 10, y, "/");
    ctx.print(x + 11, y, total);
    ctx.set_active_console(1);
}

pub fn draw_frame_bar(ctx: &mut Rltk, frame: &FrameData, x_start: u32, y_start: u32) {
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
