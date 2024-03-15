use crate::*;

pub enum SpawnType {
    Wall,
}

pub struct SpawnRequest {
    pub position: rltk::Point,
    pub spawn_type: SpawnType,
}

pub struct Spawner {
    requests: Vec<SpawnRequest>,
}

impl Spawner {
    pub fn new() -> Spawner {
        Spawner {
            requests: Vec::new(),
        }
    }

    pub fn spawn(&mut self, request: SpawnRequest) {
        self.requests.push(request);
    }
}

pub struct SpawnSystem;

impl<'a> System<'a> for SpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Spawner>,
        WriteStorage<'a, BlocksTile>,
        WriteStorage<'a, BlocksVision>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Fragile>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut spawner,
            mut blockers,
            mut vis_blockers,
            mut positions,
            mut renderables,
            mut breakables,
        ) = data;

        for request in spawner.requests.drain(..) {
            match request.spawn_type {
                SpawnType::Wall => {
                    entities
                        .build_entity()
                        .with(BlocksTile, &mut blockers)
                        .with(BlocksVision, &mut vis_blockers)
                        .with(
                            Position {
                                x: request.position.x,
                                y: request.position.y,
                            },
                            &mut positions,
                        )
                        .with(
                            Renderable {
                                symbol: rltk::to_cp437('#'),
                                fg: RGB::named(rltk::GREEN_YELLOW),
                                bg: bg_color(),
                                zindex: 1,
                            },
                            &mut renderables,
                        )
                        .with(
                            Fragile {
                                lifetime: 15,
                                was_hit: false,
                            },
                            &mut breakables,
                        )
                        .build();
                }
            }
        }
    }
}
