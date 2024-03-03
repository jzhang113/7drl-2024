use specs::prelude::*;

pub struct MapIndexSystem;

impl<'a> System<'a> for MapIndexSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, crate::Map>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, crate::Position>,
        ReadStorage<'a, crate::BlocksTile>,
        ReadStorage<'a, crate::MultiTile>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, player, positions, blockers, multis) = data;

        // Fix tiles that should be blocked
        map.set_blocked_tiles();
        map.creature_map.clear();

        for (ent, pos, _, multi) in (&entities, &positions, &blockers, (&multis).maybe()).join() {
            let index = map.get_index(pos.x, pos.y);
            map.blocked_tiles[index] = true;
            map.creature_map.insert(index, ent);

            if let Some(multi) = multi {
                for part in &multi.part_list {
                    for part_pos in part.symbol_map.keys() {
                        let part_pos_index = map.get_index(pos.x + part_pos.x, pos.y + part_pos.y);
                        map.blocked_tiles[part_pos_index] = true;
                        map.creature_map.insert(part_pos_index, ent);
                    }
                }
            }
        }

        // special handling for the player since they are not BlocksTile
        if let Some(player_pos) = positions.get(*player) {
            let player_index = map.get_index(player_pos.x, player_pos.y);
            map.creature_map.insert(player_index, *player);
        }
    }
}
