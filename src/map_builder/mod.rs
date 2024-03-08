use crate::*;
use std::collections::HashMap;

mod common;
mod noise_region;

pub mod drunk_walk;
pub mod overworld;
pub mod random_room;

const SHOW_MAPGEN_VISUALIZER: bool = false;

pub struct BuilderMap {
    pub map: Map,
    pub history: Vec<Map>,
    pub starting_position: Position,
    pub noise_areas: HashMap<i32, Vec<usize>>,
    pub rooms: Option<Vec<rltk::Rect>>,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.known_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MapBuilderArgs {
    pub width: i32,
    pub height: i32,
    pub builder_type: usize,
    pub name: String,
    pub map_color: String,
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(args: &MapBuilderArgs, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                map: Map::new(args.width, args.height, &args.name, &args.map_color, rng),
                starting_position: Position { x: 0, y: 0 },
                history: Vec::new(),
                noise_areas: HashMap::new(),
                rooms: None,
            },
        }
    }

    // I think this can return &mut Self, but idk lifetimes
    pub fn starts_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("Starting builder already exists"),
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("No starting builder"),
            Some(starter) => {
                starter.build_map(&mut self.build_data, rng);
            }
        }

        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(&mut self.build_data, rng);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World, spawn_info: &crate::SpawnInfo) {
        {
            // spawn exactly 1 of each in the major monster list
            for name in &spawn_info.major_monsters {
                let map_index = {
                    let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
                    let random_area = rng.range(0, self.build_data.noise_areas.len());
                    let random_spawn =
                        &self.build_data.noise_areas.iter().nth(random_area).unwrap();
                    random_spawn.1[rng.range(0, random_spawn.1.len())]
                };

                let entity = spawn::spawner::build_from_name(ecs, name, map_index);
                // track the entity if we built one
                if let Some(entity) = entity {
                    {
                        // mark as a target
                        let mut targets = ecs.write_storage::<crate::MissionTarget>();
                        targets.insert(entity, MissionTarget).ok();
                    }

                    spawn::spawner::track_entity(ecs, entity, map_index);
                    let mut m_info = ecs.fetch_mut::<crate::MissionInfo>();
                    m_info.add(entity);
                }
            }
        }

        // random spawns in each area of minor monsters and resources
        for area in self.build_data.noise_areas.iter() {
            spawn::spawner::spawn_region(ecs, area.1, spawn_info);
        }
    }

    pub fn spawn_overworld(&mut self, ecs: &mut World, spawn_info: &crate::SpawnInfo) {
        crate::spawn::spawner::build_npc_blacksmith(ecs, rltk::Point::new(13, 5));
        crate::spawn::spawner::build_npc_shopkeeper(ecs, rltk::Point::new(5, 5));
        crate::spawn::spawner::build_npc_handler(ecs, rltk::Point::new(13, 13));
        crate::spawn::traps::build_arrow_trap(ecs, rltk::Point::new(8, 8));
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator);
}

pub fn random_builder(width: i32, height: i32, name: String) -> BuilderChain {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder_type = rng.roll_dice(1, 3);
    println!("Building map type {}", builder_type);

    with_builder(&MapBuilderArgs {
        builder_type: builder_type as usize,
        width,
        height,
        name,
        map_color: "#FFFFFF".to_string(),
    })
}

pub fn with_builder(args: &MapBuilderArgs) -> BuilderChain {
    let mut rng = rltk::RandomNumberGenerator::new();
    let mut builder = BuilderChain::new(args, &mut rng);

    get_builder(&mut builder, args.builder_type, &mut rng);
    builder.with(noise_region::NoiseRegion::new());

    builder
}

fn get_builder(
    builder: &mut BuilderChain,
    builder_type: usize,
    rng: &mut rltk::RandomNumberGenerator,
) {
    match builder_type {
        0 => builder.starts_with(random_room::RandomRoomBuilder::new()),
        // 2 => Box::new(BspInteriorBuilder::new(new_depth)),
        // 3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        1 => builder.starts_with(drunk_walk::DrunkardsWalkBuilder::open_area()),
        2 => builder.starts_with(drunk_walk::DrunkardsWalkBuilder::open_halls()),
        3 => builder.starts_with(drunk_walk::DrunkardsWalkBuilder::winding_passages()),
        4 => builder.starts_with(overworld::OverworldBuilder::new()),
        _ => unreachable!(), //_ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
