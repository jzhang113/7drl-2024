use crate::{AttackIntent, FrameData, RangeType};
use derivative::Derivative;
use rltk::Point;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Derivative)]
#[derivative(Hash)]
pub enum AttackType {
    Melee,
    Melee2,
    Shove,
    MeleeKnockback,
    MeleeStun,
    MeleeArea { radius: i32 },
    // Projectile attacks
    Projectile { radius: i32 },
    ProjectileStun { radius: i32 },
    ProjectileKnockback { radius: i32 },
    ProjectileArea { radius: i32, explosion_size: i32 },
    OnProjectileAreaHit { radius: i32 },
    // Ranged attacks
    Ranged { radius: i32 },
    RangedStun { radius: i32 },
    RangedArea { radius: i32, explosion_size: i32 },
    // Advancing attacks
    AdvancingKnockback { range: i32 },
    AdvancingFlip { range: i32 },
    // Special cases
    Barrier,
    Hook { radius: i32 },
    Dodge { radius: i32 },
    Recover,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum AttackTrait {
    Damage { amount: i32 },
    Knockback { amount: i32 },
    Pull { amount: i32, pass_over: bool },
    Movement { delay: u32 },
    Heal { amount: i32 },
    Invulnerable { duration: u32 },
    LanceCharge { dir: crate::Direction },
    NeedsStamina { amount: i32 },
    FollowsPath { step_delay: u32, on_hit: AttackType },
    Stun { duration: u32 },
    CreatesWalls,
}

#[derive(Clone)]
pub struct AttackData {
    pub needs_target: bool,
    pub needs_path: bool,
    pub name: String,
    pub stam_cost: i32,
    pub attack_type: AttackType,
    // pub frame_data: FrameData,
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
        loc,
    }
}

pub fn get_frame_data(attack_type: AttackType) -> FrameData {
    FrameData {
        startup: get_startup(attack_type),
        active: get_active(attack_type),
        recovery: get_recovery(attack_type),
        current: 0,
        cancelled: false,
        linger_time: crate::consts::FRAME_LINGER_TIME,
    }
}

pub fn get_attack_range(attack_type: AttackType) -> RangeType {
    match attack_type {
        AttackType::Melee => RangeType::Square { size: 1 },
        AttackType::Melee2 => RangeType::Square { size: 1 },
        AttackType::Shove => RangeType::Diamond { size: 1 },
        AttackType::MeleeKnockback => RangeType::Square { size: 1 },
        AttackType::MeleeStun => RangeType::Square { size: 1 },
        AttackType::MeleeArea { .. } => RangeType::Single,
        AttackType::Projectile { radius } => RangeType::Square { size: radius },
        AttackType::ProjectileStun { radius } => RangeType::Square { size: radius },
        AttackType::ProjectileKnockback { radius } => RangeType::Square { size: radius },
        AttackType::ProjectileArea { radius, .. } => RangeType::Square { size: radius },
        AttackType::OnProjectileAreaHit { radius } => RangeType::Single,
        AttackType::Ranged { radius } => RangeType::Square { size: radius },
        AttackType::RangedStun { radius } => RangeType::Square { size: radius },
        AttackType::RangedArea { radius, .. } => RangeType::Square { size: radius },
        AttackType::AdvancingKnockback { range } => RangeType::Cross { size: range },
        AttackType::AdvancingFlip { range } => RangeType::Cross { size: range },
        AttackType::Barrier => RangeType::Single,
        AttackType::Hook { radius } => RangeType::Square { size: radius },
        AttackType::Dodge { radius } => RangeType::Diamond { size: radius },
        AttackType::Recover => RangeType::Single,
    }
}

pub fn get_attack_shape(attack_type: AttackType) -> RangeType {
    match attack_type {
        AttackType::MeleeArea { radius } => RangeType::Square { size: radius },
        AttackType::OnProjectileAreaHit { radius } => RangeType::SquareInclusive { size: radius },
        AttackType::RangedArea { explosion_size, .. } => RangeType::SquareInclusive {
            size: explosion_size,
        },
        AttackType::Barrier => RangeType::Ring { size: 3 },
        _ => RangeType::Single,
    }
}

pub fn get_startup(attack_type: AttackType) -> u32 {
    match attack_type {
        AttackType::MeleeArea { radius } => 10 + 4 * radius as u32,
        AttackType::MeleeStun => 4,
        AttackType::Shove => 3,
        AttackType::OnProjectileAreaHit { .. } => 0,
        AttackType::ProjectileStun { .. } => 16,
        AttackType::ProjectileArea { .. } => 22,
        AttackType::Ranged { .. } => 12,
        AttackType::RangedStun { .. } => 12,
        AttackType::RangedArea { .. } => 16,
        AttackType::AdvancingFlip { .. } => 6,
        AttackType::AdvancingKnockback { .. } => 6,
        AttackType::Dodge { .. } => 3,
        _ => 10,
    }
}

pub fn get_active(attack_type: AttackType) -> u32 {
    match attack_type {
        _ => 1,
    }
}

pub fn get_recovery(attack_type: AttackType) -> u32 {
    match attack_type {
        AttackType::MeleeStun => 24,
        AttackType::Ranged { .. } => 12,
        AttackType::RangedStun { .. } => 16,
        AttackType::RangedArea { .. } => 20,
        AttackType::Dodge { .. } => 5,
        _ => 10,
    }
}

use AttackTrait::*;
pub fn get_attack_traits(attack_type: AttackType) -> Vec<AttackTrait> {
    match attack_type {
        AttackType::Melee => vec![Damage { amount: 1 }],
        AttackType::Melee2 => vec![Damage { amount: 2 }],
        AttackType::MeleeKnockback => vec![Damage { amount: 1 }, Knockback { amount: 1 }],
        AttackType::Shove => vec![Knockback { amount: 3 }, NeedsStamina { amount: 3 }],
        AttackType::MeleeStun => vec![Stun { duration: 10 }],
        AttackType::MeleeArea { .. } => vec![Damage { amount: 2 }],
        AttackType::Projectile { .. } => vec![FollowsPath {
            step_delay: 3,
            on_hit: AttackType::Melee,
        }],
        AttackType::ProjectileKnockback { .. } => vec![FollowsPath {
            step_delay: 3,
            on_hit: AttackType::MeleeKnockback,
        }],
        AttackType::ProjectileStun { .. } => vec![FollowsPath {
            step_delay: 3,
            on_hit: AttackType::MeleeStun,
        }],
        AttackType::ProjectileArea { explosion_size, .. } => vec![
            FollowsPath {
                step_delay: 3,
                on_hit: AttackType::OnProjectileAreaHit {
                    radius: explosion_size,
                },
            },
            NeedsStamina {
                amount: crate::player::BOLT_STAM_REQ,
            },
        ],
        AttackType::OnProjectileAreaHit { .. } => vec![Damage { amount: 2 }],
        AttackType::Ranged { .. } => vec![Damage { amount: 1 }],
        AttackType::RangedStun { .. } => vec![Damage { amount: 1 }, Stun { duration: 10 }],
        AttackType::RangedArea { .. } => vec![Damage { amount: 1 }],
        AttackType::AdvancingKnockback { .. } => vec![
            Damage { amount: 1 },
            Knockback { amount: 2 },
            Movement { delay: 1 },
        ],
        AttackType::AdvancingFlip { .. } => vec![
            Damage { amount: 1 },
            Pull {
                amount: 2,
                pass_over: true,
            },
            Movement { delay: 1 },
            NeedsStamina {
                amount: crate::player::SUPLEX_STAM_REQ,
            },
        ],
        AttackType::Barrier => vec![CreatesWalls],
        AttackType::Hook { radius } => vec![
            Pull {
                amount: radius - 1,
                pass_over: false,
            },
            NeedsStamina {
                amount: crate::player::HOOK_STAM_REQ,
            },
        ],
        AttackType::Dodge { .. } => vec![
            Movement { delay: 0 },
            Invulnerable { duration: 6 },
            NeedsStamina {
                amount: crate::player::DODGE_STAM_REQ,
            },
        ], // 24 / 4 = 6 ticks
        AttackType::Recover => vec![Heal { amount: 2 }],
    }
}
