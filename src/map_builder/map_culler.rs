use super::{BuilderMap, MetaMapBuilder};
use crate::TileType;

pub struct MapCuller;

impl MetaMapBuilder for MapCuller {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.cull_unreachable(build_data, rng);
    }
}

impl MapCuller {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }

    fn cull_unreachable(
        &mut self,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        build_data.map.set_blocked_tiles();
        let start_idx = build_data.map.get_index(
            build_data.starting_position.x,
            build_data.starting_position.y,
        );
        let map_starts = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            build_data.map.width as usize,
            build_data.map.height as usize,
            &map_starts,
            &build_data.map,
            200.0,
        );

        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                }
            }
        }
    }
}
