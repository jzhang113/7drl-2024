use super::{BuilderMap, MetaMapBuilder};
use rltk::Rect;
use std::collections::HashSet;

pub struct NearestCorridor;

impl MetaMapBuilder for NearestCorridor {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl NearestCorridor {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }

    fn build_corridor(
        &self,
        build_data: &mut BuilderMap,
        src: rltk::Point,
        dest: rltk::Point,
    ) -> Vec<usize> {
        let mut x = src.x;
        let mut y = src.y;
        let mut tiles = Vec::new();

        while x != dest.x || y != dest.y {
            if x < dest.x {
                x += 1;
            } else if x > dest.x {
                x -= 1;
            } else if y < dest.y {
                y += 1;
            } else if y > dest.y {
                y -= 1;
            }

            let index = build_data.map.get_index(x, y);
            build_data.map.tiles[index] = crate::TileType::Floor;
            build_data.map.color_map[index] = crate::map_floor_color();
            tiles.push(index);
        }

        tiles
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        let rooms: Vec<Rect>;
        if let Some(r) = &build_data.rooms {
            rooms = r.clone();
        } else {
            panic!("No rooms to connect");
        }

        let mut connected: HashSet<usize> = HashSet::new();
        let mut corridors: Vec<Vec<usize>> = Vec::new();

        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance: Vec<(usize, f32)> = Vec::new();
            let room_center = room.center();

            for (j, other) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other.center();
                    let distance =
                        rltk::DistanceAlg::PythagorasSquared.distance2d(room_center, other_center);
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let dest_center = rooms[room_distance[0].0].center();
                let corridor = self.build_corridor(build_data, room_center, dest_center);

                connected.insert(i);
                corridors.push(corridor);
            }
        }

        build_data.corridors = Some(corridors);
    }
}
