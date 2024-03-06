use specs::prelude::*;

pub struct TrapAiSystem;

impl<'a> System<'a> for TrapAiSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::CanActFlag>,
        ReadStorage<'a, crate::Position>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::FrameData>,
        WriteStorage<'a, crate::TrapAiState>,
        ReadStorage<'a, crate::Viewshed>,
        ReadStorage<'a, crate::Moveset>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, crate::Map>,
        WriteExpect<'a, crate::ParticleBuilder>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, crate::Facing>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut can_act,
            positions,
            mut attacks,
            mut frames,
            mut states,
            viewsheds,
            movesets,
            player,
            mut map,
            mut p_builder,
            mut rng,
            facings,
        ) = data;
        let mut turn_done = Vec::new();
        let player_point = positions.get(*player).unwrap().as_point();

        for (ent, _turn, pos, state, moveset, facing) in (
            &entities,
            &can_act,
            &positions,
            &mut states,
            &movesets,
            &facings,
        )
            .join()
        {
            if let Some((atk, _)) = moveset.moves.first() {
                let dir = facing.direction.to_point();
                let target_point = rltk::Point::new(pos.x + dir.x * 8, pos.y + dir.y * 8);
                let intent = crate::attack_type::get_attack_intent(*atk, target_point, None);

                if !attacks.contains(ent) {
                    attacks.insert(ent, intent).ok();

                    frames
                        .insert(ent, crate::attack_type::get_frame_data(*atk))
                        .ok();
                } else {
                    dbg!("already an attack in queue");
                }
            }

            turn_done.push(ent);
        }

        for done in turn_done.iter() {
            can_act.remove(*done);
        }
    }
}
