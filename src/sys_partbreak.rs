use specs::prelude::*;

pub struct PartBreakSystem;

impl<'a> System<'a> for PartBreakSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, crate::Position>,
        WriteStorage<'a, crate::MultiTile>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, positions, mut multitiles) = data;

        for (_, _, multis) in (&entities, &positions, &mut multitiles).join() {
            for part in multis.part_list.iter_mut() {
                if part.health <= 0 {
                    for v in part.symbol_map.values_mut() {
                        *v = rltk::to_cp437('x');
                    }
                }
            }
        }
    }
}
