use super::consts::*;
use crate::*;

pub fn update_log_text(ecs: &World, ctx: &mut Rltk) {
    let log = ecs.fetch::<crate::gamelog::GameLog>();
    if !log.dirty {
        return;
    }

    if let Some(entry) = log.entries.last() {
        let y = CONSOLE_HEIGHT - 1;
        print_entry(ctx, y, &entry);
    }
}

fn print_entry(ctx: &mut Rltk, y: i32, entry: &crate::gamelog::LogEntry) {
    if entry.count <= 1 {
        ctx.print(1, y, &entry.text);
    } else {
        let text = format!("{} (x{})", entry.text, entry.count);
        ctx.print(1, y, text);
    }
}

pub fn expanded_log(ecs: &World, ctx: &mut Rltk) {
    let log = ecs.fetch::<crate::gamelog::GameLog>();
    let mut y = CONSOLE_HEIGHT - 10;

    ctx.set_active_console(0);
    ctx.draw_box(0, y, CONSOLE_WIDTH - 1, 10, text_color(), bg_color());
    ctx.set_active_console(1);
    ctx.draw_box(0, y, CONSOLE_WIDTH - 1, 10, text_color(), bg_color());

    y = CONSOLE_HEIGHT - 1;
    for entry in log.entries.iter().rev().take(5) {
        print_entry(ctx, y, entry);
        y -= 2;
    }
}
