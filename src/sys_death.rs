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
        ReadStorage<'a, crate::MissionTarget>,
        WriteExpect<'a, crate::MissionInfo>,
        WriteStorage<'a, crate::Fragile>,
        WriteStorage<'a, crate::Viewshed>,
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
            targets,
            mut m_info,
            mut breakables,
            mut viewsheds,
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

                    if targets.contains(ent) {
                        m_info.remove(ent);
                    }

                    if m_info.is_done() {
                        *run_state = crate::RunState::Dead { success: true };
                    }
                } else {
                    *run_state = crate::RunState::Dead { success: false };
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
