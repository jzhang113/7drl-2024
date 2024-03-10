use super::*;
use crate::*;
use rltk::{Algorithm2D, Point};
use std::collections::HashMap;

const MAX_MONSTERS: i32 = 4;

type Spawner = Box<for<'r> fn(&'r mut World, Point) -> Entity>;

lazy_static! {
    pub static ref MONSTERS: HashMap<String, (i32, Spawner)> = load_monster_table();
}

fn load_monster_table() -> HashMap<String, (i32, Spawner)> {
    let mut table = HashMap::new();

    table.insert(
        "Archer".to_string(),
        (
            1,
            Box::new(
                super::ranged::build_archer
                    as for<'r> fn(&'r mut specs::World, rltk::Point) -> specs::Entity,
            ),
        ),
    );
    table.insert(
        "Sharpshooter".to_string(),
        (2, Box::new(super::ranged::build_sharpshooter)),
    );
    table.insert(
        "Cannoneer".to_string(),
        (2, Box::new(super::ranged::build_cannoneer)),
    );
    table.insert(
        "Novice".to_string(),
        (1, Box::new(super::ranged::build_novice)),
    );
    table.insert(
        "Electromancer".to_string(),
        (2, Box::new(super::ranged::build_electromancer)),
    );
    table.insert(
        "Pyromancer".to_string(),
        (3, Box::new(super::ranged::build_pyromancer)),
    );
    table.insert(
        "Geomancer".to_string(),
        (2, Box::new(super::ranged::build_geomancer)),
    );
    table.insert(
        "Trainee".to_string(),
        (1, Box::new(super::melee::build_trainee)),
    );
    table.insert(
        "Warrior".to_string(),
        (2, Box::new(super::melee::build_warrior)),
    );
    table.insert(
        "Berserker".to_string(),
        (2, Box::new(super::melee::build_berserker)),
    );
    table.insert(
        "Juggernaut".to_string(),
        (3, Box::new(super::melee::build_juggernaut)),
    );
    table.insert(
        "Assassin".to_string(),
        (3, Box::new(super::melee::build_assassin)),
    );

    table
}

pub fn build_from_name(ecs: &mut World, name: &String, index: usize) -> Option<Entity> {
    let point = { ecs.fetch::<Map>().index_to_point2d(index) };
    MONSTERS.get(name).map(|(_, builder)| builder(ecs, point))
}

/// Fills a region with stuff!
pub fn spawn_region(ecs: &mut World, area: &[usize], difficulty: i32) -> i32 {
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);
    let mut spawns = 0;

    {
        let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
        let max_difficulty = 2 * difficulty;
        let mut curr_difficulty = 0;

        while curr_difficulty < max_difficulty {
            let rand_index = rng.range(0, super::spawner::MONSTERS.len());
            let (name, (difficulty, _)) = super::spawner::MONSTERS.iter().nth(rand_index).unwrap();
            curr_difficulty += difficulty;

            let array_index = rng.range(0, areas.len());
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, name.clone());
            areas.remove(array_index);

            // chance to early quit
            if rng.rand::<f32>() < curr_difficulty as f32 / max_difficulty as f32 {
                break;
            }
        }
    }

    // Actually spawn the monsters
    for (map_idx, name) in spawn_points.iter() {
        let entity = build_from_name(ecs, name, *map_idx);

        // track the entity if we built one
        if let Some(entity) = entity {
            track_entity(ecs, entity, *map_idx);
            spawns += 1;
        }
    }

    spawns
}

pub fn track_entity(ecs: &mut World, entity: Entity, map_idx: usize) {
    let mut map = ecs.fetch_mut::<Map>();
    let multis = ecs.read_storage::<MultiTile>();
    map.track_creature(entity, map_idx, multis.get(entity));
}

// #region Player
pub fn build_player(ecs: &mut World, point: Point) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            zindex: 1,
        })
        .with(Viewable {
            name: "Player".to_string(),
            description: vec!["That's you!".to_string()],
            seen: false,
        })
        .with(ViewableIndex { list_index: None })
        .with(Player)
        .with(Schedulable {
            current: 0,
            base: 6,
            delta: 1,
        })
        .with(Viewshed {
            visible: Vec::new(),
            dirty: true,
            range: 10,
        })
        //.with(BlocksTile)
        .with(Health {
            current: 10,
            max: 10,
        })
        .with(Stamina {
            current: 8,
            max: 8,
            recover: true,
        })
        .with(Facing {
            direction: crate::Direction::N,
        })
        .build()
}
// #endregion

// #region Enemies
pub fn build_enemy_base(ecs: &mut World) -> EntityBuilder {
    ecs.create_entity()
        .with(ViewableIndex { list_index: None })
        .with(Schedulable {
            current: 0,
            base: 6,
            delta: 1,
        })
        .with(Viewshed {
            visible: Vec::new(),
            dirty: true,
            range: 10,
        })
        .with(BlocksTile)
        .with(AiState {
            status: Behavior::Wander,
            prev_path: None,
            path_step: 0,
        })
}
// #endregion

// #region Objects
fn barrel_builder(ecs: &mut World, point: Point) -> EntityBuilder {
    ecs.create_entity()
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('#'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            zindex: 1,
        })
        .with(Viewable {
            name: "Barrel".to_string(),
            description: vec![
                "A barrel, what".to_string(),
                "could be".to_string(),
                "inside?".to_string(),
            ],
            seen: false,
        })
        .with(BlocksTile)
        .with(Openable)
        .with(Health { current: 2, max: 2 })
}

pub fn build_empty_barrel(ecs: &mut World, point: Point, _quality: i32) -> Entity {
    barrel_builder(ecs, point).build()
}

pub fn _build_health_pickup(ecs: &mut World, point: Point) -> Entity {
    ecs.create_entity()
        .with(crate::Position {
            x: point.x,
            y: point.y,
        })
        .with(crate::Renderable {
            symbol: rltk::to_cp437('+'),
            fg: crate::health_color(),
            bg: crate::bg_color(),
            zindex: 1,
        })
        .with(crate::Heal { amount: 2 })
        .with(crate::Viewable {
            name: "health".to_string(),
            description: vec!["Packaged health, don't ask".to_string()],
            seen: false,
        })
        .build()
}
// #endregion

fn build_npc_base(ecs: &mut World, point: Point) -> EntityBuilder {
    ecs.create_entity()
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(ViewableIndex { list_index: None })
        .with(Schedulable {
            current: 0,
            base: 6,
            delta: 1,
        })
        .with(Viewshed {
            visible: Vec::new(),
            dirty: true,
            range: 20,
        })
        .with(BlocksTile)
        .with(Health {
            current: 10,
            max: 10,
        })
}

pub fn build_npc_blacksmith(ecs: &mut World, point: Point) -> Entity {
    build_npc_base(ecs, point)
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            zindex: 1,
        })
        .with(Viewable {
            name: "Blacksmith".to_string(),
            description: vec!["That's you!".to_string()],
            seen: false,
        })
        .with(Npc {
            npc_type: NpcType::Blacksmith,
        })
        .build()
}

pub fn build_npc_shopkeeper(ecs: &mut World, point: Point) -> Entity {
    build_npc_base(ecs, point)
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            zindex: 1,
        })
        .with(Viewable {
            name: "Shopkeeper".to_string(),
            description: vec!["That's you!".to_string()],
            seen: false,
        })
        .with(Npc {
            npc_type: NpcType::Shopkeeper,
        })
        .build()
}

pub fn build_npc_handler(ecs: &mut World, point: Point) -> Entity {
    build_npc_base(ecs, point)
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            zindex: 1,
        })
        .with(Viewable {
            name: "Handler".to_string(),
            description: vec!["That's you!".to_string()],
            seen: false,
        })
        .with(Npc {
            npc_type: NpcType::Handler,
        })
        .build()
}
