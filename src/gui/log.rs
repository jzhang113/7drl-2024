use super::consts::*;
use crate::*;

pub fn update_log_text(ecs: &World, ctx: &mut Rltk) {
    let log = ecs.fetch::<crate::gamelog::GameLog>();
    if !log.dirty {
        return;
    }

    if let Some(strn) = log.entries.last() {
        let y = CONSOLE_HEIGHT - 1;
        ctx.print(0, y, strn);
    }
}
