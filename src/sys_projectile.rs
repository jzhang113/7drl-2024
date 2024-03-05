use rltk::Algorithm2D;
use specs::prelude::*;

pub struct ProjectileSystem;

impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, crate::Map>,
        WriteStorage<'a, crate::AttackPath>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::FrameData>,
        WriteStorage<'a, crate::Schedulable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, map, mut attack_paths, mut attacks, mut frames, mut schedulables) = data;
        let mut finished = Vec::new();

        for (ent, attack_path) in (&entities, &mut attack_paths).join() {
            attack_path.cur_delay += 1;
            if attack_path.cur_delay >= attack_path.step_delay {
                attack_path.index += 1;
                attack_path.cur_delay = 0;
            }

            if attack_path.index >= attack_path.path.len() {
                finished.push((ent, attack_path.on_hit, *attack_path.path.last().unwrap()));
                continue;
            }

            let point = attack_path.path[attack_path.index];
            let point_index = map.point2d_to_index(point);
            if let Some(aff_ent) = map.creature_map.get(&point_index) {
                // Be careful around self damage with multi-tile
                // We can't check *aff_ent != ent, since ent refers
                // to the projectile here
                finished.push((ent, attack_path.on_hit, point));
            }
        }

        for (ent, next_attack, impact_loc) in finished.iter() {
            attack_paths.remove(*ent);

            let intent = crate::AttackIntent {
                main: *next_attack,
                modifier: None,
                loc: *impact_loc,
            };

            let frame = crate::FrameData {
                startup: 0,
                active: 1,
                recovery: 1,
                current: 0,
            };

            let sched = crate::Schedulable {
                current: 0,
                base: 1,
                delta: 1,
            };

            attacks.insert(*ent, intent).ok();
            frames.insert(*ent, frame).ok();
            schedulables.insert(*ent, sched).ok();
        }
    }
}
