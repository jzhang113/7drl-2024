use super::{CanActFlag, Position, RunState, Schedulable};
use specs::prelude::*;

pub struct TurnSystem;

impl<'a> System<'a> for TurnSystem {
    type SystemData = (
        WriteExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, CanActFlag>,
        WriteStorage<'a, Schedulable>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, crate::Invulnerable>,
        WriteStorage<'a, crate::Stamina>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut game_state,
            entities,
            mut can_act,
            mut schedulables,
            pos,
            player,
            mut invulns,
            mut stams,
        ) = data;
        assert!(*game_state == RunState::Running);
        if can_act.get(*player).is_some() {
            *game_state = RunState::AwaitingInput;
            return;
        }

        let mut invuln_over = Vec::new();

        for (ent, sched, _pos, invuln, stam) in (
            &entities,
            &mut schedulables,
            &pos,
            (&mut invulns).maybe(),
            (&mut stams).maybe(),
        )
            .join()
        {
            sched.current -= sched.delta;

            if let Some(invuln) = invuln {
                invuln.duration -= 1;

                if invuln.duration <= 0 {
                    invuln_over.push(ent);
                }
            }

            if sched.current > 0 {
                continue;
            }

            if let Some(stamina) = stam {
                if !stamina.recover {
                    stamina.recover = true;
                } else if stamina.current < stamina.max {
                    stamina.current += 1;
                }
            }

            sched.current += sched.base;
            can_act
                .insert(
                    ent,
                    CanActFlag {
                        is_reaction: false,
                        reaction_target: None,
                    },
                )
                .expect("Failed to insert CanActFlag");
        }

        for done in invuln_over {
            invulns.remove(done);
        }
    }
}
