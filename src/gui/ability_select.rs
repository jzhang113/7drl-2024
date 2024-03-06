use super::consts::*;
use crate::*;

pub fn draw_abilities(gs: &State, ctx: &mut Rltk, selected_idx: usize) {
    let book_x = MAP_SCREEN_X + 1;
    let book_y = MAP_SCREEN_Y + 1;
    let box_w = 15;
    let box_h = gs.player_abilities.len() * 2 + 1;

    ctx.draw_box(
        book_x,
        book_y,
        box_w,
        box_h,
        RGB::named(rltk::GREY),
        RGB::named(rltk::BLACK),
    );

    let header = "Use Ability";
    ctx.print(book_x + 1, book_y, header);

    for (i, ability) in gs.player_abilities.iter().enumerate() {
        let row = book_y + 2 + 2 * i as i32;
        let mut text_color = crate::text_color();

        if i == selected_idx {
            ctx.set_active_console(0);
            for dx in 0..box_w - 1 {
                ctx.set_bg(book_x + 1 + dx, row, crate::select_highlight_color());
            }
            ctx.set_active_console(1);

            text_color = crate::select_text_color();
        }

        let cdx = char::from_u32(i as u32 + 97).unwrap();
        ctx.print_color(book_x + 1, row, text_color, bg_color(), cdx);
        ctx.print_color(book_x + 2, row, text_color, bg_color(), '-');
        ctx.print_color(
            book_x + 3,
            row,
            text_color,
            bg_color(),
            ability.name.clone(),
        );
    }
}
