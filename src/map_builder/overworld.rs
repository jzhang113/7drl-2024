use rltk::Algorithm2D;

use super::MapBuilder;
use crate::*;

pub struct OverworldBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
}

impl MapBuilder for OverworldBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) -> Map {
        self.build(rng);
        self.get_map()
    }

    fn spawn_entities(&mut self, ecs: &mut World, spawn_info: &crate::SpawnInfo) {
        add_npc(ecs);
        spawn::traps::build_arrow_trap(ecs, rltk::Point::new(8, 8));
    }

    fn take_snapshot(&mut self) {
        if super::SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.known_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl OverworldBuilder {
    pub fn new(args: &map_builder::MapBuilderArgs, rng: &mut rltk::RandomNumberGenerator) -> Self {
        Self {
            map: Map::new(args.width, args.height, &args.name, &args.map_color, rng),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
        }
    }

    fn build(&mut self, _rng: &mut rltk::RandomNumberGenerator) {
        self.starting_position = Position {
            x: self.map.width / 2 - 1,
            y: self.map.height / 2 - 1,
        };

        let diameter = std::cmp::min(self.map.width, self.map.height) - 2;
        let r_squared = (diameter * diameter) as f32 / 4.0;

        for x in 0..self.map.width {
            for y in 0..self.map.height {
                if rltk::DistanceAlg::PythagorasSquared
                    .distance2d(self.starting_position.as_point(), rltk::Point::new(x, y))
                    < r_squared
                {
                    let index = self.map.get_index(x, y);
                    self.map.tiles[index] = TileType::Floor;
                }
            }
        }

        for y in (self.map.height * 2 / 5)..(self.map.height * 3 / 5) {
            let exit_index = self.map.get_index(self.map.width - 2, y);
            self.map.tiles[exit_index] = TileType::NewLevel;

            let exit_index = self.map.get_index(self.map.width - 1, y);
            self.map.tiles[exit_index] = TileType::Floor;
        }

        for x in 13..=14 {
            for y in 7..=10 {
                let index = self.map.get_index(x, y);
                self.map.tiles[index] = TileType::Water;
            }
        }

        self.take_snapshot();
    }
}

fn add_npc(ecs: &mut World) {
    crate::spawn::spawner::build_npc_blacksmith(ecs, rltk::Point::new(13, 5));
    crate::spawn::spawner::build_npc_shopkeeper(ecs, rltk::Point::new(5, 5));
    crate::spawn::spawner::build_npc_handler(ecs, rltk::Point::new(13, 13));
}
