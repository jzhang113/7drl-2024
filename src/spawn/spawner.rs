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
        "Pusher".to_string(),
        (
            1,
            Box::new(build_mook as for<'r> fn(&'r mut specs::World, rltk::Point) -> specs::Entity),
        ),
    );
    table.insert("Archer".to_string(), (2, Box::new(build_archer)));
    table.insert("Crab".to_string(), (1, Box::new(build_crab)));

    table
}

pub fn build_from_name(ecs: &mut World, name: &String, index: usize) -> Option<Entity> {
    let point = { ecs.fetch::<Map>().index_to_point2d(index) };
    MONSTERS.get(name).map(|(_, builder)| builder(ecs, point))
}

/// Fills a region with stuff!
pub fn spawn_region(ecs: &mut World, area: &[usize], spawn_info: &crate::SpawnInfo) {
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    {
        let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_MONSTERS + 3) + (4 - 1) - 3,
        );

        if num_spawns == 0 {
            return;
        }

        for _i in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };

            let map_idx = areas[array_index];
            if let Some(spawn) = roll(spawn_info, &mut *rng) {
                spawn_points.insert(map_idx, spawn);
                areas.remove(array_index);
            }
        }
    }

    // Actually spawn the monsters
    for (map_idx, name) in spawn_points.iter() {
        //let point = map.index_to_point2d(*map_idx);
        let entity = build_from_name(ecs, name, *map_idx);
        // track_entity(ecs, &mut *map, entity, point);
        // spawn_entity(ecs, &spawn);

        // track the entity if we built one
        if let Some(entity) = entity {
            track_entity(ecs, entity, *map_idx);
        }
    }
}

pub fn track_entity(ecs: &mut World, entity: Entity, map_idx: usize) {
    let mut map = ecs.fetch_mut::<Map>();
    let multis = ecs.read_storage::<MultiTile>();
    map.track_creature(entity, map_idx, multis.get(entity));
}

fn roll(spawn_info: &SpawnInfo, rng: &mut rltk::RandomNumberGenerator) -> Option<String> {
    let type_roll = rng.rand::<f32>();

    if type_roll < 0.25 {
        if spawn_info.minor_monsters.len() == 0 {
            return None;
        }

        let roll = rng.range::<usize>(0, spawn_info.minor_monsters.len());
        Some(spawn_info.minor_monsters[roll].clone())
    } else {
        if spawn_info.resources.len() == 0 {
            return None;
        }

        let roll = rng.range::<usize>(0, spawn_info.resources.len());
        Some(spawn_info.resources[roll].clone())
    }
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
            range: 20,
        })
        .with(CanReactFlag)
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
            base: 24,
            delta: 4,
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

pub fn build_mook(ecs: &mut World, point: Point) -> Entity {
    let part_list = vec![
        MonsterPart {
            symbol_map: HashMap::from([
                (rltk::Point::new(-1, 0), rltk::to_cp437('│')),
                (rltk::Point::new(-1, 1), rltk::to_cp437('└')),
                (rltk::Point::new(0, 1), rltk::to_cp437('─')),
            ]),
            health: 4,
            max_health: 4,
        },
        MonsterPart {
            symbol_map: HashMap::from([
                (rltk::Point::new(0, -1), rltk::to_cp437('─')),
                (rltk::Point::new(1, -1), rltk::to_cp437('┐')),
                (rltk::Point::new(1, 0), rltk::to_cp437('│')),
            ]),
            health: 4,
            max_health: 4,
        },
    ];

    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('x'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewable {
            name: "Pusher".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health {
            current: 10,
            max: 10,
        })
        .with(Moveset {
            moves: vec![(AttackType::Area, 0.25), (AttackType::Melee, 0.75)],
            bump_attack: AttackType::Melee,
        })
        .with(MultiTile {
            bounds: all_bounds(&part_list),
            part_list: part_list,
        })
        .with(Facing {
            direction: crate::Direction::N,
        })
        .build()
}

pub fn build_crab(ecs: &mut World, point: Point) -> Entity {
    let part_list = vec![
        MonsterPart {
            symbol_map: HashMap::from([
                (rltk::Point::new(1, 0), rltk::to_cp437('─')),
                (rltk::Point::new(2, 1), rltk::to_cp437('\\')),
            ]),
            health: 1,
            max_health: 1,
        },
        MonsterPart {
            symbol_map: HashMap::from([
                (rltk::Point::new(-1, 0), rltk::to_cp437('─')),
                (rltk::Point::new(-2, 1), rltk::to_cp437('/')),
            ]),
            health: 1,
            max_health: 1,
        },
    ];

    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('x'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewable {
            name: "Crab".to_string(),
            description: vec![],
            seen: false,
        })
        .with(Health {
            current: 10,
            max: 10,
        })
        .with(Moveset {
            moves: vec![(AttackType::Area, 0.25), (AttackType::Melee, 0.75)],
            bump_attack: AttackType::Melee,
        })
        .with(MultiTile {
            bounds: all_bounds(&part_list),
            part_list: part_list,
        })
        .with(Facing {
            direction: crate::Direction::N,
        })
        .build()
}

pub fn build_archer(ecs: &mut World, point: Point) -> Entity {
    let part_list = vec![
        MonsterPart {
            symbol_map: HashMap::from([(rltk::Point::new(-1, 0), rltk::to_cp437('<'))]),
            health: 2,
            max_health: 2,
        },
        MonsterPart {
            symbol_map: HashMap::from([(rltk::Point::new(0, 1), rltk::to_cp437('v'))]),
            health: 2,
            max_health: 2,
        },
        MonsterPart {
            symbol_map: HashMap::from([(rltk::Point::new(1, 0), rltk::to_cp437('>'))]),
            health: 2,
            max_health: 2,
        },
        MonsterPart {
            symbol_map: HashMap::from([(rltk::Point::new(0, -1), rltk::to_cp437('^'))]),
            health: 2,
            max_health: 2,
        },
    ];

    build_enemy_base(ecs)
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('y'),
            fg: RGB::named(rltk::LIGHT_GREEN),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewable {
            name: "Archer".to_string(),
            description: vec!["A grunt with a bow".to_string()],
            seen: false,
        })
        .with(Health { current: 6, max: 6 })
        .with(MultiTile {
            bounds: all_bounds(&part_list),
            part_list: part_list,
        })
        .with(Moveset {
            moves: vec![
                (AttackType::Melee, 0.25),
                (AttackType::Bolt { radius: 6 }, 0.75),
            ],
            bump_attack: AttackType::Melee,
        })
        .build()
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
            base: 24,
            delta: 4,
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
