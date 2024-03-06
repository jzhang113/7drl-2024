use specs::prelude::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, crate::Map>,
        ReadStorage<'a, crate::Moveset>,
        ReadStorage<'a, crate::MultiTile>,
        WriteStorage<'a, crate::Position>,
        WriteStorage<'a, crate::MoveIntent>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::FrameData>,
        WriteStorage<'a, crate::Viewshed>,
        WriteStorage<'a, crate::Facing>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            movesets,
            multis,
            mut positions,
            mut movements,
            mut attacks,
            mut frames,
            mut viewsheds,
            mut facings,
        ) = data;

        for (ent, pos, movement, moveset, multi, viewshed, facing) in (
            &entities,
            &mut positions,
            &movements,
            (&movesets).maybe(),
            (&multis).maybe(),
            (&mut viewsheds).maybe(),
            (&mut facings).maybe(),
        )
            .join()
        {
            let new_pos = movement.loc;
            let mut attack_pos = None;

            if let Some(facing) = facing {
                if let Some(dir) = movement.force_facing {
                    facing.direction = dir;
                } else if let Some(dir) =
                    crate::Direction::get_direction_towards(pos.as_point(), new_pos)
                {
                    facing.direction = dir;
                }
            }

            // check for the player at the destination, since we should already be pathing around other entities
            if let Some(multi) = multi {
                attack_pos = check_for_entity_at(ent, &mut map, multi, new_pos);
            }

            match attack_pos {
                Some(attack_pos) => {
                    if let Some(moveset) = moveset {
                        attacks
                            .insert(
                                ent,
                                crate::attack_type::get_attack_intent(
                                    moveset.bump_attack,
                                    attack_pos,
                                    None,
                                ),
                            )
                            .ok();

                        frames
                            .insert(ent, crate::attack_type::get_frame_data(moveset.bump_attack))
                            .ok();
                    }
                }
                None => {
                    // update the position if we successfully moved to new_pos
                    if map.move_creature(ent, pos.as_point(), new_pos, multi) {
                        pos.x = new_pos.x;
                        pos.y = new_pos.y;

                        if let Some(viewshed) = viewshed {
                            viewshed.dirty = true;
                        }
                    }
                }
            }
        }

        movements.clear();
    }
}

fn check_for_entity_at(
    ent: Entity,
    map: &mut crate::Map,
    multi: &crate::MultiTile,
    pos: rltk::Point,
) -> Option<rltk::Point> {
    for part in &multi.part_list {
        for part_pos in part.symbol_map.keys() {
            let new_x = pos.x + part_pos.x;
            let new_y = pos.y + part_pos.y;
            let map_idx = map.get_index(new_x, new_y);

            if let Some(creature_at) = map.creature_map.get(&map_idx) {
                if ent != *creature_at {
                    return Some(rltk::Point::new(new_x, new_y));
                }
            }
        }
    }

    None
}
