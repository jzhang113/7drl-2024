use specs::prelude::*;

pub struct StunSystem;

impl<'a> System<'a> for StunSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::Stunned>,
        WriteStorage<'a, crate::Schedulable>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::AttackInProgress>,
        WriteStorage<'a, crate::FrameData>,
        ReadStorage<'a, crate::Position>,
        ReadStorage<'a, crate::Viewable>,
        WriteExpect<'a, crate::GameLog>,
        WriteExpect<'a, crate::Map>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut stuns,
            mut schedulables,
            mut attacks,
            mut attacks_in_progress,
            mut frames,
            positions,
            viewables,
            mut log,
            mut map,
            player,
        ) = data;
        let mut removal = Vec::new();

        for (ent, stun, sched, pos, view) in
            (&entities, &stuns, &mut schedulables, &positions, &viewables).join()
        {
            removal.push(ent);

            if let Some(attack) = attacks.get(ent) {
                sched.current = stun.duration as i32;

                if map.visible_tiles[map.get_index(pos.x, pos.y)] {
                    if ent == *player {
                        log.add("Your attack is interrupted");
                    } else {
                        log.add(format!(
                            "A {}'s attack is interrupted",
                            view.name.to_lowercase()
                        ));
                    }
                }
            } else {
                sched.current += stun.duration as i32;
            }
        }

        for done in removal.iter() {
            stuns.remove(*done);
            attacks.remove(*done);
            attacks_in_progress.remove(*done);

            if let Some(frame) = frames.get_mut(*done) {
                frame.cancelled = true;
            }
        }
    }
}
