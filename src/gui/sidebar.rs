use super::consts::*;
use crate::weapon::WeaponButton;
use crate::*;
use rltk::{Rltk, RGB};

pub fn draw_sidebar(gs: &State, ctx: &mut Rltk) {
    let players = gs.ecs.read_storage::<Player>();
    let healths = gs.ecs.read_storage::<Health>();
    let stams = gs.ecs.read_storage::<Stamina>();
    let views = gs.ecs.read_storage::<Viewable>();

    let player = gs.ecs.fetch::<Entity>();
    let m_info = gs.ecs.fetch::<MissionInfo>();
    let next_state = gs.ecs.fetch::<RunState>();

    ctx.draw_box(
        SIDE_X,
        SIDE_Y,
        SIDE_W,
        SIDE_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let x = SIDE_X + 1;
    let mut y = SIDE_Y + 1;

    for (_, stamina, health) in (&players, &stams, &healths).join() {
        draw_resource_bar(
            ctx,
            health.current,
            health.max,
            x,
            y,
            hp_main_color(),
            hp_alt_color(),
        );

        draw_resource_bar(
            ctx,
            stamina.current,
            stamina.max,
            x,
            y + 1,
            stam_main_color(),
            stam_alt_color(),
        );
    }

    if gs.player_charging.0 {
        ctx.print(x, y + 2, "Charging!");
    }

    // Quest info
    y += 4;
    ctx.print(x, y, "Quest:");

    if let Some(quest) = &gs.selected_quest {
        if quest.started {
            match *next_state {
                RunState::Dead { success } => {
                    if success {
                        ctx.print_color(
                            x + 6,
                            y,
                            crate::text_success_color(),
                            crate::bg_color(),
                            "Complete!",
                        );
                    } else {
                        ctx.print_color(
                            x + 6,
                            y,
                            crate::text_failed_color(),
                            crate::bg_color(),
                            "Failed",
                        );
                    }

                    ctx.print_color(x, y + 2, text_highlight_color(), bg_color(), "r");
                    ctx.print(x + 1, y + 2, "eturn to base");
                }
                _ => {
                    if !m_info.is_done() {
                        ctx.print_color(
                            x + 6,
                            y,
                            crate::text_highlight_color(),
                            crate::bg_color(),
                            "In prgrss",
                        );
                        ctx.print(x, y + 2, "Remaining");
                        for (i, ent) in m_info.remaining.iter().enumerate() {
                            if let Some(ent_view) = views.get(*ent) {
                                ctx.print(x + 2, y + 4 + 2 * i as i32, ent_view.name.clone());
                            }
                        }
                    }
                }
            }
        } else {
            ctx.print_color(
                x + 6,
                y,
                crate::text_highlight_color(),
                crate::bg_color(),
                "Accepted",
            );
            ctx.print(x, y + 2, "Targets");
            for (i, name) in quest.spawn_info.major_monsters.iter().enumerate() {
                ctx.print(x + 2, y + 4 + 2 * i as i32, name);
            }
        }
    } else {
        ctx.print_color(
            x + 6,
            y,
            crate::text_failed_color(),
            crate::bg_color(),
            "None",
        );
    }

    // Resources
    y = 30;
    ctx.print(x, y, format!("Money:{}z", gs.player_inventory.money));
    ctx.print(
        x,
        y + 2,
        format!("Weapon:{}", gs.player_inventory.weapon.name()),
    );
    ctx.print(
        x,
        y + 4,
        format!("Armor:+{}", gs.player_inventory.armor_level),
    );

    // Weapon info
    y = 38;
    ctx.print(x, y, "Controls");
    draw_movement_controls(ctx, x, y + 2, text_highlight_color(), bg_color(), false);

    let dodge_icon_color = if crate::player::can_dodge(&gs) {
        text_highlight_color()
    } else {
        text_inactive_color()
    };
    ctx.print_color(x, y + 4, dodge_icon_color, bg_color(), "[SPACE]");
    ctx.print(x + 8, y + 4, "Dodge");

    let player_stam = stams.get(*player).unwrap().current;
    add_weapon_text(ctx, x, y + 6, &gs.player_inventory.weapon, player_stam);

    super::tooltip::draw_tooltips(&gs.ecs, ctx);
}

fn draw_resource_bar(
    ctx: &mut Rltk,
    curr: i32,
    max: i32,
    x: i32,
    y: i32,
    main_color: RGB,
    alt_color: RGB,
) {
    let curr = std::cmp::max(0, curr);
    for i in 0..curr {
        ctx.set(x + i, y, main_color, bg_color(), rltk::to_cp437('o'));
    }

    for i in curr..max {
        ctx.set(x + i, y, alt_color, bg_color(), rltk::to_cp437('o'));
    }
}

fn draw_movement_controls(ctx: &mut Rltk, x: i32, y: i32, fg: RGB, bg: RGB, inactive: bool) {
    ctx.set(x, y, fg, bg, 27);
    ctx.set(x + 1, y, fg, bg, 25);
    ctx.set(x + 2, y, fg, bg, 24);
    ctx.set(x + 3, y, fg, bg, 26);

    if inactive {
        ctx.print_color(x + 8, y, fg, bg, "Move");
    } else {
        ctx.print(x + 8, y, "Move");
    }
}

fn add_weapon_text(
    ctx: &mut Rltk,
    x: i32,
    y: i32,
    weapon: &Box<dyn crate::weapon::Weapon>,
    player_stam: i32,
) {
    let icon_color = text_highlight_color();
    let inactive_color = text_inactive_color();
    let bg_color = bg_color();

    let mut y = y;

    if let Some(name) = weapon.attack_name(WeaponButton::Light) {
        let sheathe_icon_color = if name != "Draw Atk" {
            icon_color
        } else {
            inactive_color
        };

        ctx.print_color(x, y, sheathe_icon_color, bg_color, "s");
        ctx.print(x + 2, y, "Sheathe");

        if let Some(stam_cost) = weapon.can_activate_cost(WeaponButton::Light) {
            let attack_icon_color = if stam_cost <= player_stam {
                icon_color
            } else {
                inactive_color
            };

            y += 2;
            ctx.print_color(x, y, attack_icon_color, bg_color, "z");
            ctx.print(x + 2, y, &name);
        }
    }

    if let Some(name) = weapon.attack_name(WeaponButton::Heavy) {
        if let Some(stam_cost) = weapon.can_activate_cost(WeaponButton::Heavy) {
            let attack_icon_color = if stam_cost <= player_stam {
                icon_color
            } else {
                inactive_color
            };

            y += 2;
            ctx.print_color(x, y, attack_icon_color, bg_color, "x");
            ctx.print(x + 2, y, &name);
        }
    }

    if let Some(name) = weapon.attack_name(WeaponButton::Special) {
        if let Some(stam_cost) = weapon.can_activate_cost(WeaponButton::Special) {
            let attack_icon_color = if stam_cost <= player_stam {
                icon_color
            } else {
                inactive_color
            };

            y += 2;
            ctx.print_color(x, y, attack_icon_color, bg_color, "c");
            ctx.print(x + 2, y, &name);
        }
    }
}
