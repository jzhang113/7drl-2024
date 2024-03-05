use specs::prelude::*;

pub struct StunSystem;

impl<'a> System<'a> for StunSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::Stunned>,
        WriteStorage<'a, crate::Schedulable>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::AttackInProgress>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut stuns, mut schedulables, mut attacks, mut attacks_in_progress) = data;
        let mut removal = Vec::new();

        for (ent, stun, sched) in (&entities, &stuns, &mut schedulables).join() {
            removal.push(ent);

            if let Some(attack) = attacks.get(ent) {
                sched.current = stun.duration as i32;
            } else {
                sched.current += stun.duration as i32;
            }
        }

        for done in removal.iter() {
            stuns.remove(*done);
            attacks.remove(*done);
            attacks_in_progress.remove(*done);
        }
    }
}
