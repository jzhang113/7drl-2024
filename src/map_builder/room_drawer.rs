use super::{BuilderMap, MetaMapBuilder};
use crate::TileType;
use rltk::Rect;

pub struct RoomDrawerSettings {
    pub circle_chance: f32,
}

pub struct RoomDrawer {
    settings: RoomDrawerSettings,
}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl RoomDrawer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            settings: RoomDrawerSettings {
                circle_chance: 0.25,
            },
        })
    }

    fn rect_room(&self, build_data: &mut BuilderMap, room: &Rect) {
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let index = build_data.map.get_index(x, y);
                build_data.map.tiles[index] = TileType::Floor;
                build_data.map.color_map[index] = crate::map_floor_color();
            }
        }
    }

    fn circle_room(&self, build_data: &mut BuilderMap, room: &Rect) {
        let center = room.center();
        let diameter = std::cmp::min(room.width(), room.height()) - 2;
        let r2 = (diameter * diameter) as f32 / 4.0;

        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let index = build_data.map.get_index(x, y);
                let distance =
                    rltk::DistanceAlg::PythagorasSquared.distance2d(center, rltk::Point::new(x, y));

                if distance < r2 {
                    build_data.map.tiles[index] = TileType::Floor;
                    build_data.map.color_map[index] = crate::map_floor_color();
                }
            }
        }
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        let rooms: Vec<Rect>;
        if let Some(r) = &build_data.rooms {
            rooms = r.clone();
        } else {
            panic!("No rooms to build");
        }

        for room in rooms.iter() {
            if rng.rand::<f32>() < self.settings.circle_chance {
                self.circle_room(build_data, room);
            } else {
                self.rect_room(build_data, room);
            }
        }
    }
}
