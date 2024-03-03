use specs::prelude::*;
use std::collections::HashMap;

pub struct PartMoveSystem;

impl<'a> System<'a> for PartMoveSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, crate::Position>,
        WriteStorage<'a, crate::PartMoveIntent>,
        WriteStorage<'a, crate::MultiTile>,
        WriteStorage<'a, crate::PushForce>,
        ReadExpect<'a, crate::Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, positions, mut part_moves, mut multitiles, mut pushes, map) = data;

        for (ent, ent_pos, moves, multis) in
            (&entities, &positions, &part_moves, &mut multitiles).join()
        {
            let mut pushed_ents = Vec::new();

            for (i, dir) in moves.part_delta.iter().enumerate() {
                let mut new_symbol_map = HashMap::new();

                for (part_pos, symbol) in &multis.part_list[i].symbol_map {
                    let new_pos = *part_pos + *dir;
                    new_symbol_map.insert(new_pos, *symbol);

                    if let Some(map_ent) = map
                        .creature_map
                        .get(&map.get_index(ent_pos.x + new_pos.x, ent_pos.y + new_pos.y))
                    {
                        if *map_ent != ent && !pushed_ents.contains(map_ent) {
                            // each entity can only be pushed once
                            pushed_ents.push(*map_ent);
                            pushes
                                .insert(*map_ent, crate::PushForce { delta: *dir })
                                .expect("Failed to insert push caused by part movement");
                        }
                    }
                }

                multis.part_list[i].symbol_map = new_symbol_map;
            }
        }

        part_moves.clear();
    }
}
