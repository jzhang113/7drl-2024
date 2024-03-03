use crate::quest::*;
use rltk::{Rltk, RGB};

pub fn draw_missions(
    ctx: &mut Rltk,
    quest_log: &log::QuestLog,
    current_quest: &Option<quest::Quest>,
    selected_idx: usize,
) {
    let book_x = 8;
    let book_y = 6;
    let book_page_w = 40;
    let book_page_h = 40;

    ctx.draw_box(
        book_x,
        book_y,
        book_page_w,
        book_page_h,
        RGB::named(rltk::GREY),
        RGB::named(rltk::BLACK),
    );
    ctx.draw_box(
        book_x + book_page_w,
        book_y,
        book_page_w,
        book_page_h,
        RGB::named(rltk::GREY),
        RGB::named(rltk::BLACK),
    );

    let header = "Select Quest";
    ctx.print(
        book_x + (book_page_w - header.len()) / 2,
        book_y + 2,
        header,
    );

    for (i, quest) in quest_log.entries.iter().enumerate() {
        let row = book_y + 4 + 2 * i;
        let mut text_color = crate::text_color();

        if i == selected_idx {
            ctx.set_active_console(0);
            for dx in 0..book_page_w - 1 {
                ctx.set_bg(book_x + 1 + dx, row, crate::select_highlight_color());
            }
            ctx.set_active_console(1);

            text_color = crate::select_text_color();
        }

        if current_quest.as_ref().map_or(false, |q| q == quest) {
            let assigned_str = "Assigned!";
            ctx.print_color(
                book_x + book_page_w - assigned_str.len(),
                row,
                text_color,
                crate::bg_color(),
                assigned_str,
            );
        } else if quest.completed {
            let complete_str = "Completed!";
            ctx.print_color(
                book_x + book_page_w - complete_str.len(),
                row,
                crate::text_success_color(),
                crate::bg_color(),
                complete_str,
            );
        } else {
            let remaining_str = format!("{} days left", quest.days_remaining);
            ctx.print_color(
                book_x + book_page_w - remaining_str.len(),
                row,
                text_color,
                crate::bg_color(),
                remaining_str,
            );
        }

        let mut quest_name = quest.get_name();
        let quest_max_len = book_page_w - 11 - 2;
        if quest_name.len() > quest_max_len {
            quest_name.truncate(quest_max_len);
            quest_name.replace_range((quest_max_len - 3)..quest_max_len, "...");
        }
        ctx.print_color(book_x + 1, row, text_color, crate::bg_color(), quest_name);
    }

    if selected_idx < quest_log.entries.len() {
        let quest = &quest_log.entries[selected_idx];
        ctx.print(
            book_x + 1,
            book_y + book_page_h - 7,
            quest.quest_type.name(),
        );
        ctx.print(book_x + 1, book_y + book_page_h - 5, "Hunt all targets");
        ctx.print(
            book_x + 1,
            book_y + book_page_h - 3,
            format!("Reward Money: {}z", quest.reward),
        );
        ctx.print(
            book_x + 1,
            book_y + book_page_h - 1,
            format!("Time Limit: {} turns", quest.turn_limit),
        );

        ctx.print(
            book_x + book_page_w + 2,
            book_y + 2,
            quest.area_name.clone(),
        );

        ctx.print(
            book_x + book_page_w + 2,
            book_y + book_page_h - 2 * quest.spawn_info.major_monsters.len() - 1,
            "Targets:",
        );
        for (i, name) in quest.spawn_info.major_monsters.iter().enumerate() {
            ctx.print(
                book_x + book_page_w + 3,
                book_y + book_page_h - 2 * i - 1,
                name,
            );
        }
    }
}

pub fn draw_upgrades(ctx: &mut Rltk) {
    let book_x = 8;
    let book_y = 6;
    let box_w = 40;
    let box_h = 40;

    ctx.draw_box(
        book_x,
        book_y,
        box_w,
        box_h,
        RGB::named(rltk::GREY),
        RGB::named(rltk::BLACK),
    );

    let header = "Placeholder";
    ctx.print(book_x + (box_w - header.len()) / 2, book_y + 2, header);
}

pub fn draw_shop(ctx: &mut Rltk) {
    let book_x = 8;
    let book_y = 6;
    let box_w = 40;
    let box_h = 40;

    ctx.draw_box(
        book_x,
        book_y,
        box_w,
        box_h,
        RGB::named(rltk::GREY),
        RGB::named(rltk::BLACK),
    );

    let header = "Placeholder";
    ctx.print(book_x + (box_w - header.len()) / 2, book_y + 2, header);
}
