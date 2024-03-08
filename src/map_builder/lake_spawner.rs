use super::{BuilderMap, MetaMapBuilder};
use crate::TileType;
use std::cmp::{max, min};

pub struct LakeSpawnerSettings {
    pub lake_chance: f32,
    pub lake_erosion: f32,
    pub max_lakes: u32,
}

pub struct LakeSpawner {
    settings: LakeSpawnerSettings,
}

impl MetaMapBuilder for LakeSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl LakeSpawner {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            settings: LakeSpawnerSettings {
                lake_chance: 0.3,
                lake_erosion: 0.01,
                max_lakes: 3,
            },
        })
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        if !build_data.noise_areas.is_empty() {
            // ensure there is at least one non-lake area
            let mut lake_count = 0;
            let max_lakes = min(
                self.settings.max_lakes,
                (build_data.noise_areas.len() - 1).try_into().unwrap(),
            );
            let mut removal = Vec::new();

            for (key, region) in build_data.noise_areas.iter() {
                let lake_prob = rng.rand::<f32>();
                if lake_prob < self.settings.lake_chance {
                    lake_count += 1;
                    removal.push(*key);

                    for t in region {
                        build_data.map.tiles[*t] = TileType::Water;
                    }
                }

                if lake_count >= max_lakes {
                    break;
                }
            }

            for key in removal {
                build_data.noise_areas.remove(&key);
            }
        } else {
            todo!("Adding lakes only implemented with existing noise regions");
        }
    }
}

pub struct LakeEroderSettings {
    pub deep_erosion_chance: f32,
    pub shallow_erosion_chance: f32,
    pub erosion_rounds: u32,
}

pub struct LakeEroder {
    settings: LakeEroderSettings,
}

impl MetaMapBuilder for LakeEroder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl LakeEroder {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            settings: LakeEroderSettings {
                deep_erosion_chance: 0.3,
                shallow_erosion_chance: 0.15,
                erosion_rounds: 10,
            },
        })
    }

    fn neighbor_indices(map: &crate::Map, x: i32, y: i32) -> Vec<usize> {
        let xmin = max(x - 1, 0);
        let xmax = min(x + 1, map.width);
        let ymin = max(y - 1, 0);
        let ymax = min(y + 1, map.height);
        let mut indices = Vec::new();

        for xs in xmin..=xmax {
            for ys in ymin..=ymax {
                indices.push(map.get_index(xs, ys));
            }
        }

        indices
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        let max_index = (build_data.map.width * build_data.map.height) as usize;

        for _ in 0..self.settings.erosion_rounds {
            for x in 0..build_data.map.width {
                for y in 0..build_data.map.height {
                    let idx = build_data.map.get_index(x, y);
                    if build_data.map.tiles[idx] == TileType::Water {
                        let nidix = Self::neighbor_indices(&build_data.map, x, y);
                        let water_neighbors = nidix
                            .iter()
                            .filter(|ndx| build_data.map.tiles[**ndx] == TileType::Water)
                            .count();

                        let erosion_prob = rng.rand::<f32>();
                        if water_neighbors < 7 && erosion_prob < self.settings.deep_erosion_chance {
                            build_data.map.tiles[idx] = TileType::ShallowWater;
                        }
                    }

                    if build_data.map.tiles[idx] == TileType::ShallowWater {
                        let nidix = Self::neighbor_indices(&build_data.map, x, y);
                        let water_neighbors = nidix
                            .iter()
                            .filter(|ndx| {
                                build_data.map.tiles[**ndx] == TileType::Water
                                    || build_data.map.tiles[**ndx] == TileType::ShallowWater
                            })
                            .count();

                        let erosion_prob = rng.rand::<f32>();
                        if water_neighbors < 7
                            && erosion_prob < self.settings.shallow_erosion_chance
                        {
                            build_data.map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }
}
