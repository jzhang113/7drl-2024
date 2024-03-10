use specs::prelude::*;

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, crate::Map>,
        WriteExpect<'a, crate::RunState>,
        ReadStorage<'a, crate::Position>,
        ReadStorage<'a, crate::Health>,
        ReadStorage<'a, crate::MultiTile>,
        WriteStorage<'a, crate::Fragile>,
        WriteStorage<'a, crate::Viewshed>,
        WriteExpect<'a, crate::GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player,
            mut map,
            mut run_state,
            positions,
            healths,
            multitiles,
            breakables,
            mut viewsheds,
            mut log,
        ) = data;
        let mut dead = Vec::new();

        for (ent, pos, health, multis) in
            (&entities, &positions, &healths, (&multitiles).maybe()).join()
        {
            let pos_index = map.get_index(pos.x, pos.y);

            if health.current <= 0 {
                if ent != *player {
                    dead.push(ent);
                    map.untrack_creature(pos_index, multis);
                } else {
                    *run_state = crate::RunState::Dead { success: false };
                    log.add("You are knocked out! Press r to try again")
                }
            }
        }

        for (ent, pos, fragile) in (&entities, &positions, &breakables).join() {
            if fragile.was_hit || fragile.lifetime == 0 {
                let pos_index = map.get_index(pos.x, pos.y);
                map.untrack_creature(pos_index, None);
                dead.push(ent);
            }
        }

        // if any dead entities blocked vision, we need to recompute viewsheds
        for (ent, viewshed) in (&entities, &mut viewsheds).join() {
            viewshed.dirty = true;
        }

        for victim in dead {
            entities
                .delete(victim)
                .expect("Failed to remove dead entity");
        }
    }
}
