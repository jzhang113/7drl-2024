use super::consts::*;
use crate::*;

pub fn draw_frames(ecs: &World, ctx: &mut Rltk) {
    let attacks = ecs.read_storage::<AttackIntent>();
    let frames = ecs.read_storage::<FrameData>();
    let players = ecs.read_storage::<Player>();

    let xx = 17;
    let yy = 49;

    for (_, frame) in (&players, &frames).join() {
        for x in 0..frame.startup {
            ctx.draw_box(
                xx + x,
                yy,
                1,
                1,
                RGB::named(rltk::GREEN),
                RGB::named(rltk::BLACK),
            );
        }

        for x in 0..frame.active {
            ctx.draw_box(
                xx + x + frame.startup,
                yy,
                1,
                1,
                RGB::named(rltk::ORANGE),
                RGB::named(rltk::BLACK),
            );
        }

        for x in 0..frame.recovery {
            ctx.draw_box(
                xx + x + frame.startup + frame.active,
                yy,
                1,
                1,
                RGB::named(rltk::BLUE),
                RGB::named(rltk::BLACK),
            );
        }

        ctx.draw_box(
            xx + frame.current,
            yy,
            1,
            1,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }
}
