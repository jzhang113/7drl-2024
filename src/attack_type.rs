use crate::{AttackIntent, RangeType};
use derivative::Derivative;
use rltk::Point;
use std::collections::HashMap;

lazy_static! {
    static ref STARTUP_ACTIONS: HashMap<AttackType, Vec<crate::NextIntent>> = startup_actions();
    static ref RECOVERY_ACTIONS: HashMap<AttackType, Vec<crate::NextIntent>> = recovery_actions();
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Derivative)]
#[derivative(Hash)]
pub enum AttackType {
    Sweep,
    Punch,
    Stun,
    Push,
    Dodge,
    Recover,
    // lance
    LanceDraw,
    LanceThrust { level: u8, dest: Point },
    LanceCharge { dir: crate::Direction },
    LanceSweep,
    // enemy specific attacks
    Haymaker,
    Ranged,
}

#[derive(PartialEq, Copy, Clone)]
pub enum AttackTrait {
    Damage { amount: i32 },
    Knockback { amount: i32 },
    Movement,
    Heal { amount: i32 },
    Invulnerable { duration: u32 },
    LanceCharge { dir: crate::Direction },
    NeedsStamina { amount: i32 },
}

// check if an attack is can be executed
// this returns the tile that will hit the target
pub fn is_attack_valid(attack_type: AttackType, from_point: Point, target: Point) -> Option<Point> {
    let range_type = get_attack_range(attack_type);
    let shape = get_attack_shape(attack_type);

    for tile in crate::range_type::resolve_range_at(&range_type, from_point) {
        let affected_tiles = crate::range_type::resolve_range_at(&shape, tile);

        if affected_tiles.contains(&target) {
            return Some(tile);
        }
    }

    None
}

// return all points that are affected by an attack
pub fn each_attack_target(attack_type: AttackType, from_point: Point) -> Vec<Point> {
    let shape = get_attack_shape(attack_type);
    crate::range_type::resolve_range_at(&shape, from_point)
}

// convert an attack into an intent that can be executed by the event system
pub fn get_attack_intent(
    attack_type: AttackType,
    loc: Point,
    attack_modifier: Option<AttackType>,
) -> AttackIntent {
    AttackIntent {
        main: attack_type,
        modifier: attack_modifier,
        loc,
        delay: get_attack_delay(attack_type),
    }
}

pub fn get_attack_range(attack_type: AttackType) -> RangeType {
    match attack_type {
        AttackType::Sweep => RangeType::Single,
        AttackType::Punch => RangeType::Square { size: 1 },
        AttackType::Stun => RangeType::Square { size: 1 },
        AttackType::Push => RangeType::Square { size: 1 },
        AttackType::Dodge => RangeType::Diamond { size: 2 },
        AttackType::Recover => RangeType::Single,
        AttackType::Haymaker => RangeType::Square { size: 1 },
        AttackType::Ranged => RangeType::Square { size: 3 },
        AttackType::LanceDraw => RangeType::Square { size: 1 },
        AttackType::LanceThrust { .. } => RangeType::Square { size: 1 },
        AttackType::LanceCharge { .. } => RangeType::Single,
        AttackType::LanceSweep => RangeType::Square { size: 1 },
    }
}

pub fn get_attack_power(attack_type: AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 1,
        AttackType::Punch => 1,
        AttackType::Stun => 0,
        AttackType::Push => 0,
        AttackType::Dodge => 0,
        AttackType::Recover => 0,
        AttackType::Haymaker => 3,
        AttackType::Ranged => 1,
        AttackType::LanceDraw => 1,
        AttackType::LanceThrust { level, .. } => level as i32,
        AttackType::LanceCharge { .. } => 0,
        AttackType::LanceSweep => 2,
    }
}

pub fn get_attack_shape(attack_type: AttackType) -> RangeType {
    match attack_type {
        AttackType::Sweep => RangeType::Square { size: 3 },
        AttackType::Punch => RangeType::Single,
        AttackType::Stun => RangeType::Single,
        AttackType::Push => RangeType::Single,
        AttackType::Dodge => RangeType::Single,
        AttackType::Recover => RangeType::Single,
        AttackType::Haymaker => RangeType::Single,
        AttackType::Ranged => RangeType::Single,
        AttackType::LanceDraw => RangeType::Single,
        AttackType::LanceThrust { dest, .. } => RangeType::Path { dest },
        AttackType::LanceCharge { .. } => RangeType::Single,
        AttackType::LanceSweep => RangeType::Square { size: 1 },
    }
}

pub fn get_attack_delay(attack_type: AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 2,
        AttackType::Punch => 0,
        AttackType::Stun => 2,
        AttackType::Push => 0,
        AttackType::Recover => 0,
        AttackType::Haymaker => -4,
        AttackType::Ranged => 0,
        _ => 0,
    }
}

pub fn get_startup(attack_type: AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 1,
        _ => 0,
    }
}

pub fn get_recovery(attack_type: AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 1,
        AttackType::Ranged => 1,
        _ => 0,
    }
}

pub fn get_startup_action(attack_type: AttackType, index: usize) -> crate::NextIntent {
    match STARTUP_ACTIONS.get(&attack_type) {
        Some(action_list) => action_list[index % action_list.len()].clone(),
        None => crate::NextIntent::None,
    }
}

pub fn get_recovery_action(attack_type: AttackType, index: usize) -> crate::NextIntent {
    match RECOVERY_ACTIONS.get(&attack_type) {
        Some(action_list) => action_list[index % action_list.len()].clone(),
        None => crate::NextIntent::None,
    }
}

fn startup_actions() -> HashMap<AttackType, Vec<crate::NextIntent>> {
    let mut action_map = HashMap::new();

    action_map.insert(
        AttackType::Sweep,
        vec![crate::NextIntent::PartMove {
            intent: crate::PartMoveIntent {
                part_delta: vec![rltk::Point::new(-1, 1), rltk::Point::new(1, -1)],
            },
        }],
    );

    action_map
}

fn recovery_actions() -> HashMap<AttackType, Vec<crate::NextIntent>> {
    let mut action_map = HashMap::new();

    action_map.insert(
        AttackType::Sweep,
        vec![crate::NextIntent::PartMove {
            intent: crate::PartMoveIntent {
                part_delta: vec![rltk::Point::new(1, -1), rltk::Point::new(-1, 1)],
            },
        }],
    );

    action_map
}

pub fn get_attack_name(attack_type: AttackType) -> String {
    match attack_type {
        AttackType::Sweep => "sweep",
        AttackType::Punch => "punch",
        AttackType::Stun => "stun",
        AttackType::Push => "push",
        AttackType::Dodge => "dodge",
        AttackType::Recover => "recover",
        AttackType::Haymaker => "haymaker",
        AttackType::Ranged => "shoot",
        AttackType::LanceDraw => "Draw Atk",
        AttackType::LanceThrust { .. } => "Thrust",
        AttackType::LanceCharge { .. } => "Charge",
        AttackType::LanceSweep => "Sweep",
    }
    .to_string()
}

use AttackTrait::*;
pub fn get_attack_traits(attack_type: AttackType) -> Vec<AttackTrait> {
    match attack_type {
        AttackType::Sweep => vec![Damage { amount: 2 }],
        AttackType::Punch => vec![Damage { amount: 1 }],
        AttackType::Stun => vec![Damage { amount: 0 }],
        AttackType::Push => vec![Knockback { amount: 2 }],
        AttackType::Dodge => vec![
            Movement,
            Invulnerable { duration: 6 },
            NeedsStamina {
                amount: crate::player::DODGE_STAM_REQ,
            },
        ], // 24 / 4 = 6 ticks
        AttackType::Recover => vec![Heal { amount: 2 }],
        AttackType::Haymaker => vec![Damage { amount: 2 }],
        AttackType::Ranged => vec![Damage { amount: 1 }],
        AttackType::LanceDraw => vec![Damage { amount: 1 }],
        AttackType::LanceThrust { level, .. } => vec![Damage {
            amount: level as i32,
        }],
        AttackType::LanceCharge { dir } => vec![LanceCharge { dir }],
        AttackType::LanceSweep => vec![Damage { amount: 2 }],
    }
}
