use super::spawner::build_enemy_base;
use crate::*;
use rltk::Point;

pub fn build_archer(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('a'),
            fg: enemy_ranged_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Archer".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 2, max: 2 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::Projectile { radius: 5 }, 0.75),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}

pub fn build_sharpshooter(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('s'),
            fg: enemy_ranged_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Sharpshooter".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 4, max: 4 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::ProjectileStun { radius: 5 }, 0.75),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}

pub fn build_cannoneer(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('c'),
            fg: enemy_ranged_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Cannoneer".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 4, max: 4 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (
                    AttackType::ProjectileArea {
                        radius: 5,
                        explosion_size: 2,
                    },
                    0.75,
                ),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}

pub fn build_novice(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('n'),
            fg: enemy_mage_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Novice".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 2, max: 2 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::Ranged { radius: 7 }, 0.75),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}

pub fn build_electromancer(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('e'),
            fg: enemy_mage_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Electromancer".to_string(),
            description: vec!["A grunt with a bow".to_string()],
            seen: false,
        })
        .with(Health { current: 4, max: 4 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::RangedStun { radius: 5 }, 0.75),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}

pub fn build_pyromancer(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('p'),
            fg: enemy_mage_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Pyromancer".to_string(),
            description: vec!["A grunt with a bow".to_string()],
            seen: false,
        })
        .with(Health { current: 4, max: 4 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (
                    AttackType::RangedArea {
                        radius: 7,
                        explosion_size: 3,
                    },
                    0.75,
                ),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}

pub fn build_geomancer(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('g'),
            fg: enemy_mage_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Geomancer".to_string(),
            description: vec!["A grunt with a bow".to_string()],
            seen: false,
        })
        .with(Health { current: 6, max: 6 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::Barrier, 0.75),
            ],
            bump_attack: AttackType::MeleeKnockback,
        })
        .build()
}
