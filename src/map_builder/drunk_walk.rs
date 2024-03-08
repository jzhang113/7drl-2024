use super::common::*;
use super::{BuilderMap, InitialMapBuilder};
use crate::*;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub drunken_lifetime: i32,
    pub floor_percent: f32,
    pub digger_size: i32,
}

pub struct DrunkardsWalkBuilder {
    settings: DrunkardSettings,
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl DrunkardsWalkBuilder {
    #[allow(dead_code)]
    pub fn new(settings: DrunkardSettings) -> Self {
        Self { settings }
    }

    pub fn open_area() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 1000,
                floor_percent: 0.5,
                digger_size: 4,
            },
        })
    }

    pub fn open_halls() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                digger_size: 3,
            },
        })
    }

    pub fn winding_passages() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                digger_size: 2,
            },
        })
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        // Set a central starting point
        build_data.starting_position = Position {
            x: build_data.map.width / 2,
            y: build_data.map.height / 2,
        };
        let start_idx = build_data.map.get_index(
            build_data.starting_position.x,
            build_data.starting_position.y,
        );
        build_data.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut digger_count = 0;
        let mut active_digger_count = 0;
        while floor_tile_count < desired_floor_tiles && digger_count < 1000 {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = build_data.starting_position.x;
                    drunk_y = build_data.starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = build_data.starting_position.x;
                        drunk_y = build_data.starting_position.y;
                    } else {
                        drunk_x = rng
                            .roll_dice(1, build_data.map.width - self.settings.digger_size - 2)
                            + 1;
                        drunk_y = rng
                            .roll_dice(1, build_data.map.height - self.settings.digger_size - 2)
                            + 1;
                    }
                }
            }
            let mut drunk_life = self.settings.drunken_lifetime;

            while drunk_life > 0 {
                for dx in 0..=self.settings.digger_size {
                    for dy in 0..=self.settings.digger_size {
                        let drunk_idx = build_data.map.get_index(drunk_x + dx, drunk_y + dy);
                        if build_data.map.tiles[drunk_idx] == TileType::Wall {
                            did_something = true;
                        }
                        build_data.map.tiles[drunk_idx] = TileType::DownStairs;
                    }
                }

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => {
                        if drunk_x > 2 {
                            drunk_x -= 1;
                        }
                    }
                    2 => {
                        if drunk_x < build_data.map.width - self.settings.digger_size - 2 {
                            drunk_x += 1;
                        }
                    }
                    3 => {
                        if drunk_y > 2 {
                            drunk_y -= 1;
                        }
                    }
                    _ => {
                        if drunk_y < build_data.map.height - self.settings.digger_size - 2 {
                            drunk_y += 1;
                        }
                    }
                }

                drunk_life -= 1;
            }
            if did_something {
                build_data.take_snapshot();
                active_digger_count += 1;
            }

            digger_count += 1;
            for t in build_data.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = build_data
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }
        rltk::console::log(format!(
            "{} dwarves gave up their sobriety, of whom {} actually found a wall.",
            digger_count, active_digger_count
        ));

        // Find all tiles we can reach from the starting point
        let exit_tile =
            remove_unreachable_areas_returning_most_distant(&mut build_data.map, start_idx);
        build_data.take_snapshot();

        // Place the stairs
        // build_data.map.tiles[exit_tile] = TileType::DownStairs;
        build_data.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        build_data.noise_areas = generate_voronoi_spawn_regions(&build_data.map, rng);
    }
}
