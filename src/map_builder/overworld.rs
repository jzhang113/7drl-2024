use super::{BuilderMap, InitialMapBuilder};
use crate::*;

pub struct OverworldBuilder;

impl InitialMapBuilder for OverworldBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl OverworldBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        build_data.starting_position = Position {
            x: build_data.map.width / 2 - 1,
            y: build_data.map.height / 2 - 1,
        };

        let diameter = std::cmp::min(build_data.map.width, build_data.map.height) - 2;
        let r_squared = (diameter * diameter) as f32 / 4.0;

        for x in 0..build_data.map.width {
            for y in 0..build_data.map.height {
                if rltk::DistanceAlg::PythagorasSquared.distance2d(
                    build_data.starting_position.as_point(),
                    rltk::Point::new(x, y),
                ) < r_squared
                {
                    let index = build_data.map.get_index(x, y);
                    build_data.map.tiles[index] = TileType::Floor;
                }
            }
        }

        for y in (build_data.map.height * 2 / 5)..(build_data.map.height * 3 / 5) {
            let exit_index = build_data.map.get_index(build_data.map.width - 2, y);
            build_data.map.tiles[exit_index] = TileType::NewLevel;

            let exit_index = build_data.map.get_index(build_data.map.width - 1, y);
            build_data.map.tiles[exit_index] = TileType::Floor;
        }

        build_data.take_snapshot();
    }
}
