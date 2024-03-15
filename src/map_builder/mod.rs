use crate::*;
use std::collections::HashMap;

mod common;
mod lake_spawner;
mod map_culler;
mod noise_region;
mod room_corridor;
mod room_drawer;
mod starting_pos;

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
    pub corridors: Option<Vec<Vec<usize>>>,
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
    pub level: u32,
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
                map: Map::new(
                    args.width,
                    args.height,
                    args.level,
                    &args.name,
                    &args.map_color,
                    rng,
                ),
                starting_position: Position { x: 0, y: 0 },
                history: Vec::new(),
                noise_areas: HashMap::new(),
                rooms: None,
                corridors: None,
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

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        let mut count = 0;

        // random spawns in each area of minor monsters and resources
        for area in self.build_data.noise_areas.iter() {
            count += spawn::spawner::spawn_region(ecs, area.1, self.build_data.map.level as i32);
        }

        let mut map = ecs.fetch_mut::<Map>();
        map.initial_spawns = count;
    }

    pub fn spawn_overworld(&mut self, ecs: &mut World) {
        crate::spawn::spawner::build_npc_blacksmith(ecs, rltk::Point::new(13, 5));
        crate::spawn::spawner::build_npc_shopkeeper(ecs, rltk::Point::new(5, 5));
        crate::spawn::spawner::build_npc_handler(ecs, rltk::Point::new(9, 3));
        // crate::spawn::traps::build_arrow_trap(ecs, rltk::Point::new(8, 8));
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator);
}

pub fn random_builder(width: i32, height: i32, level: u32, name: String) -> BuilderChain {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder_type = rng.range(0, 5);
    println!("Building map type {}", builder_type);

    with_builder(&MapBuilderArgs {
        builder_type: builder_type as usize,
        width,
        height,
        level,
        name,
        map_color: "#FFFFFF".to_string(),
    })
}

pub fn with_builder(args: &MapBuilderArgs) -> BuilderChain {
    let mut rng = rltk::RandomNumberGenerator::new();
    let mut builder = BuilderChain::new(args, &mut rng);

    get_builder(&mut builder, args.builder_type, &mut rng);

    if args.builder_type != 99 {
        builder.with(noise_region::NoiseRegion::new());
        builder.with(lake_spawner::LakeSpawner::new());
        builder.with(lake_spawner::LakeEroder::new());
        builder.with(map_culler::MapCuller::new());

        // refresh noise regions for spawn placements
        builder.with(noise_region::NoiseRegion::new());
    }

    builder
}

fn get_builder(
    builder: &mut BuilderChain,
    builder_type: usize,
    rng: &mut rltk::RandomNumberGenerator,
) {
    match builder_type {
        0 | 1 => {
            builder.starts_with(random_room::RandomRoomBuilder::new());
            builder.with(room_drawer::RoomDrawer::new());
            builder.with(room_corridor::NearestCorridor::new());
            builder.with(drunk_walk::DrunkardsWalkBuilder::winding_passages());
            builder.with(starting_pos::RoomBasedStartingPos::new());
        }
        // 2 => Box::new(BspInteriorBuilder::new(new_depth)),
        // 3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        2 => builder.starts_with(drunk_walk::DrunkardsWalkBuilder::open_area()),
        3 => builder.starts_with(drunk_walk::DrunkardsWalkBuilder::open_halls()),
        4 => builder.starts_with(drunk_walk::DrunkardsWalkBuilder::winding_passages()),
        99 => builder.starts_with(overworld::OverworldBuilder::new()),
        _ => unreachable!(), //_ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
