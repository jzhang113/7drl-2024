use super::consts::*;
use crate::*;

pub fn update_log_text(ecs: &World, ctx: &mut Rltk) {
    let log = ecs.fetch::<crate::gamelog::GameLog>();
    if !log.dirty {
        return;
    }

    if let Some(entry) = log.entries.last() {
        let y = CONSOLE_HEIGHT - 1;

        if entry.count <= 1 {
            ctx.print(0, y, &entry.text);
        } else {
            let text = format!("{} (x{})", entry.text, entry.count);
            ctx.print(0, y, text);
        }
    }
}
