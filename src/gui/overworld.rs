use rltk::{Rltk, RGB};

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
