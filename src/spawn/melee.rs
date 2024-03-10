use super::spawner::build_enemy_base;
use crate::*;
use rltk::Point;

pub fn build_trainee(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('t'),
            fg: enemy_melee_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Trainee".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 3, max: 3 })
        .with(Moveset {
            moves: vec![(AttackType::Melee, 1.0)],
            bump_attack: AttackType::Melee,
        })
        .build()
}

pub fn build_warrior(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('w'),
            fg: enemy_melee_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Warrior".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 6, max: 6 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeStun, 0.75),
                (AttackType::MeleeKnockback, 0.25),
            ],
            bump_attack: AttackType::Melee,
        })
        .build()
}

pub fn build_berserker(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('b'),
            fg: enemy_melee_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Berserker".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 6, max: 6 })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::MeleeArea { radius: 3 }, 0.75),
            ],
            bump_attack: AttackType::Melee2,
        })
        .build()
}

pub fn build_juggernaut(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('j'),
            fg: enemy_melee_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Juggernaut".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health {
            current: 10,
            max: 10,
        })
        .with(Moveset {
            moves: vec![
                (AttackType::MeleeKnockback, 0.25),
                (AttackType::AdvancingKnockback { range: 2 }, 0.25),
                (AttackType::AdvancingFlip { range: 2 }, 0.25),
                (AttackType::MeleeArea { radius: 2 }, 0.75),
            ],
            bump_attack: AttackType::Melee2,
        })
        .build()
}

pub fn build_assassin(ecs: &mut World, point: Point) -> Entity {
    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('y'),
            fg: enemy_melee_color(),
            bg: bg_color(),
            zindex: 1,
        })
        .with(Viewable {
            name: "Assassin".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health { current: 3, max: 3 })
        .with(Moveset {
            moves: vec![
                (AttackType::Hook { radius: 4 }, 0.5),
                (AttackType::Melee2, 0.5),
            ],
            bump_attack: AttackType::Melee2,
        })
        .build()
}
