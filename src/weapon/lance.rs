use crate::{AttackData, AttackIntent, AttackTrait, AttackType};

#[derive(Copy, Clone)]
pub enum LanceAttack {
    DrawAttack,
    Thrust { level: u8 },
    Charge,
    Sweep,
}

pub fn get_attack_data(attack: LanceAttack) -> AttackData {
    let needs_target = match attack {
        LanceAttack::Sweep => true,
        LanceAttack::DrawAttack => true,
        _ => false,
    };

    let needs_path = match attack {
        LanceAttack::Sweep => true,
        _ => false,
    };

    let name = match attack {
        LanceAttack::DrawAttack => "Draw Atk",
        LanceAttack::Thrust { level } => match level {
            1 => "Stab I",
            2 => "Stab II",
            3 => "Stab III",
            4 => "Charge Stab",
            _ => unreachable!(),
        },
        LanceAttack::Charge => "Charge",
        LanceAttack::Sweep => "Sweep",
    }
    .to_string();

    let stam_cost = match attack {
        LanceAttack::DrawAttack => 1,
        LanceAttack::Thrust { level } => match level {
            1 => 2,
            2 => 3,
            3 => 3,
            4 => 0,
            _ => unreachable!(),
        },
        LanceAttack::Charge => 2,
        LanceAttack::Sweep => 3,
    };

    let attack_type = match attack {
        LanceAttack::Sweep => AttackType::Bolt { radius: 6 },
        LanceAttack::DrawAttack => AttackType::Advancing,
        LanceAttack::Thrust { level } => AttackType::Melee,
        LanceAttack::Charge => AttackType::Melee,
    };

    let traits = match attack {
        LanceAttack::Sweep => vec![AttackTrait::FollowsPath {
            step_delay: 3,
            on_hit: AttackType::Area,
        }],
        LanceAttack::DrawAttack => vec![AttackTrait::Damage { amount: 1 }],
        LanceAttack::Thrust { level } => vec![AttackTrait::Damage {
            amount: level as i32,
        }],
        LanceAttack::Charge => vec![AttackTrait::Damage { amount: 1 }],
    };

    let frame_data = crate::FrameData {
        startup: 15,
        active: 1,
        recovery: 15,
        current: 0,
        cancelled: false,
        linger_time: 10,
    };

    AttackData {
        needs_target,
        needs_path,
        name,
        stam_cost,
        attack_type,
        traits,
        frame_data,
    }
}

fn get_attack_intent(
    attack: LanceAttack,
    from_point: rltk::Point,
    dir: crate::Direction,
) -> AttackIntent {
    let source_point = crate::direction::Direction::point_in_direction(from_point, dir);

    match attack {
        LanceAttack::DrawAttack => AttackIntent {
            main: AttackType::Advancing,
            loc: source_point,
        },
        LanceAttack::Thrust { level } => AttackIntent {
            main: AttackType::LanceThrust {
                level,
                dest: crate::direction::Direction::point_in_direction(source_point, dir),
            },
            loc: from_point,
        },
        LanceAttack::Charge => AttackIntent {
            main: AttackType::LanceCharge { dir },
            loc: from_point,
        },
        LanceAttack::Sweep => AttackIntent {
            main: AttackType::Bolt { radius: 4 },
            loc: from_point,
        },
    }
}
