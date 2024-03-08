use super::{BuilderMap, MetaMapBuilder};

pub struct RoomBasedStartingPos;

impl MetaMapBuilder for RoomBasedStartingPos {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.build(build_data, rng);
    }
}

impl RoomBasedStartingPos {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }

    fn build(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        if let Some(rooms) = &build_data.rooms {
            let start_pos = rooms[0].center();
            build_data.starting_position = crate::Position {
                x: start_pos.x,
                y: start_pos.y,
            };
        } else {
            panic!("No rooms, could not set starting position");
        }
    }
}
