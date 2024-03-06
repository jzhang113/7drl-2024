use specs::prelude::*;

pub struct FrameDataSystem;

impl<'a> System<'a> for FrameDataSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::FrameData>,
        WriteStorage<'a, crate::Schedulable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut frames, mut schedulables) = data;
        let mut finished = Vec::new();

        for (ent, frame, sched) in (&entities, &mut frames, &mut schedulables).join() {
            if !frame.cancelled {
                frame.current += 1;
                sched.current += 1;

                if frame.current >= frame.startup + frame.active + frame.recovery {
                    finished.push(ent);
                }
            } else {
                frame.linger_time -= 1;
                if frame.linger_time <= 0 {
                    finished.push(ent);
                }
            }
        }

        for done in finished.iter() {
            frames.remove(*done);
        }
    }
}
