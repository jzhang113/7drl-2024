use super::{BuilderMap, MetaMapBuilder};
use crate::TileType;
use noise::*;
use std::collections::HashMap;

pub struct NoiseRegion;

impl MetaMapBuilder for NoiseRegion {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.generate_voronoi_spawn_regions(build_data, rng);
    }
}

impl NoiseRegion {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }

    /// Generates a Voronoi/cellular noise map of a region, and divides it into spawn regions.
    #[allow(clippy::map_entry)]
    pub fn generate_voronoi_spawn_regions(
        &mut self,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
        let generator = Worley::new(rng.roll_dice(1, 65536) as u32)
            .set_frequency(0.05)
            .set_return_type(core::worley::ReturnType::Value);

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let idx = build_data.map.get_index(x, y);
                if build_data.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = generator.get([x as f64, y as f64]) * 1024.0;
                    let cell_value = cell_value_f as i32;

                    if noise_areas.contains_key(&cell_value) {
                        noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }

        build_data.noise_areas = noise_areas
    }
}
