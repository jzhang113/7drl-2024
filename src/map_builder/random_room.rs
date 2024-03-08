use super::{BuilderMap, InitialMapBuilder};
use rltk::Rect;

pub struct RandomRoomSettings {
    pub max_rooms: i32,
    pub min_room_width: i32,
    pub max_room_width: i32,
    pub min_room_height: i32,
    pub max_room_height: i32,
}

pub struct RandomRoomBuilder {
    settings: RandomRoomSettings,
}

impl InitialMapBuilder for RandomRoomBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build_rooms(build_data, rng);
    }
}

impl RandomRoomBuilder {
    pub fn new() -> Box<RandomRoomBuilder> {
        Box::new(Self {
            settings: RandomRoomSettings {
                max_rooms: 12,
                min_room_width: 5,
                max_room_width: 15,
                min_room_height: 5,
                max_room_height: 15,
            },
        })
    }

    fn build_rooms(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        let mut rooms: Vec<rltk::Rect> = Vec::new();

        for _ in 0..self.settings.max_rooms {
            let w = rng.range(self.settings.min_room_width, self.settings.max_room_width);
            let h = rng.range(self.settings.min_room_height, self.settings.max_room_height);
            let x = rng.range(1, build_data.map.width - w - 1);
            let y = rng.range(1, build_data.map.height - h - 1);

            let new_room = Rect::with_size(x, y, w, h);
            let mut quit = false;

            for other_rooms in rooms.iter() {
                if other_rooms.intersect(&new_room) {
                    quit = true;
                }
            }

            if !quit {
                rooms.push(new_room);
            }
        }

        for r in rooms.iter() {
            build_data.map.build_room(*r);
        }
        build_data.map.set_blocked_tiles();

        build_data.rooms = Some(rooms);
    }
}
