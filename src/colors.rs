use rltk::{RandomNumberGenerator, HSV, RGB};

pub fn bg_color() -> RGB {
    RGB::named(rltk::BLACK)
}

pub fn attack_highlight_color() -> RGB {
    RGB::named(rltk::LIGHTBLUE)
}

pub fn hp_main_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn hp_alt_color() -> RGB {
    RGB::named(rltk::DARKRED)
}

pub fn stam_main_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn stam_alt_color() -> RGB {
    RGB::named(rltk::DARKGOLDENROD)
}

pub fn select_highlight_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn select_text_color() -> RGB {
    RGB::named(rltk::BLACK)
}

pub fn text_highlight_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn text_color() -> RGB {
    RGB::named(rltk::WHITE)
}

pub fn text_inactive_color() -> RGB {
    RGB::named(rltk::GREY)
}

pub fn text_failed_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn text_success_color() -> RGB {
    RGB::named(rltk::GREEN)
}

pub fn map_floor_color() -> RGB {
    let hsv = HSV::from_f32(0.3, 0.2, 0.2);
    hsv.to_rgb()
}

pub fn map_wall_variant(base_color: HSV, rng: &mut RandomNumberGenerator) -> RGB {
    let hue_change = 0.12 * (rng.rand::<f32>() - 0.5);
    let sat_change = 0.2 * (rng.rand::<f32>() - 0.5);
    let hsv = HSV::from_f32(
        base_color.h + hue_change,
        base_color.s + sat_change,
        base_color.v,
    );
    hsv.to_rgb()
}

pub fn attack_source_color() -> RGB {
    RGB::named(rltk::LIGHTBLUE)
}

pub fn attack_target_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn valid_cursor_color() -> RGB {
    RGB::named(rltk::CYAN)
}

pub fn invalid_cursor_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn attack_intent_color() -> RGB {
    RGB::named(rltk::PURPLE)
}

pub fn tiles_in_range_color() -> RGB {
    RGB::named(rltk::BLUE)
}

pub fn header_message_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn header_err_color() -> RGB {
    RGB::named(rltk::DARKGOLDENROD)
}

pub fn particle_hit_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn particle_bg_color() -> RGB {
    RGB::named(rltk::DARKRED)
}

pub fn health_color() -> RGB {
    RGB::named(rltk::MAGENTA)
}

pub fn map_exit_color() -> RGB {
    RGB::named(rltk::WHITE)
}

pub fn map_water_color() -> RGB {
    RGB::named(rltk::DARKBLUE)
}

pub fn frame_startup_color() -> RGB {
    RGB::named(rltk::GREEN)
}

pub fn frame_active_color() -> RGB {
    RGB::named(rltk::ORANGE)
}

pub fn frame_recovery_color() -> RGB {
    RGB::named(rltk::BLUE)
}

pub fn frame_current_color() -> RGB {
    RGB::named(rltk::RED)
}
