use super::{BuilderMap, MetaMapBuilder};

pub struct OverworldSpawner;

impl MetaMapBuilder for OverworldSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        self.spawn_entities(build_data, rng);
    }
}

impl OverworldSpawner {
    pub fn new() -> Box<OverworldSpawner> {
        Box::new(OverworldSpawner)
    }

    fn spawn_entities(&mut self, ecs: &mut World, spawn_info: &crate::SpawnInfo) {
        add_npc(ecs);
    }
}

fn add_npc(ecs: &mut World) {
    crate::spawn::spawner::build_npc_blacksmith(ecs, rltk::Point::new(13, 5));
    crate::spawn::spawner::build_npc_shopkeeper(ecs, rltk::Point::new(5, 5));
    crate::spawn::spawner::build_npc_handler(ecs, rltk::Point::new(8, 5));
}
