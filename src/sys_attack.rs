use crate::attack_type;
use rltk::Algorithm2D;
use specs::prelude::*;
use std::collections::HashMap;

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, crate::Map>,
        ReadStorage<'a, crate::Position>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::AttackInProgress>,
        WriteStorage<'a, crate::Health>,
        WriteStorage<'a, crate::MultiTile>,
        WriteExpect<'a, crate::ParticleBuilder>,
        WriteExpect<'a, crate::Spawner>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, crate::RunState>,
        WriteStorage<'a, crate::Invulnerable>,
        WriteStorage<'a, crate::MoveIntent>,
        WriteStorage<'a, crate::Stamina>,
        WriteStorage<'a, crate::AttackPath>,
        ReadStorage<'a, crate::FrameData>,
        WriteStorage<'a, crate::Stunned>,
        WriteStorage<'a, crate::Fragile>,
        ReadStorage<'a, crate::Viewable>,
        WriteExpect<'a, crate::GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            positions,
            mut attacks,
            mut attacks_in_progress,
            mut healths,
            mut multis,
            mut p_builder,
            mut s_builder,
            player,
            mut run_state,
            mut invulns,
            mut movements,
            mut stams,
            mut attack_paths,
            frames,
            mut stuns,
            mut breakables,
            viewables,
            mut log,
        ) = data;
        let mut finished_attacks = Vec::new();

        for (ent, intent, frame) in (&entities, &mut attacks, &frames).join() {
            let trait_list = attack_type::get_attack_traits(intent.main);

            if frame.current <= frame.startup {
                if !trait_list
                    .iter()
                    .any(|tr| matches!(tr, attack_type::AttackTrait::FollowsPath { .. }))
                {
                    attacks_in_progress
                        .insert(ent, crate::AttackInProgress)
                        .expect("Failed to insert AttackInProgress flag");
                }

                continue;
            }

            if frame.current >= frame.startup + frame.active {
                finished_attacks.push(ent);
            }

            for att_trait in trait_list {
                match att_trait {
                    crate::AttackTrait::Knockback { amount } => {
                        let ents_hit = self.get_hit_entities(&mut p_builder, &map, ent, intent);
                        for (ent_hit, _) in ents_hit {
                            let src_pos = positions.get(ent).unwrap().as_point();
                            let ent_pos = positions.get(ent_hit).unwrap().as_point();

                            let offset = crate::Direction::get_direction_towards(src_pos, ent_pos)
                                .unwrap()
                                .to_point();

                            // check for collision
                            let mut next_x = ent_pos.x;
                            let mut next_y = ent_pos.y;
                            for _ in 0..amount {
                                next_x += offset.x;
                                next_y += offset.y;

                                // if we collide into something, rewind the attempted movement
                                // and insert a stun
                                if !map.is_tile_valid(next_x, next_y) {
                                    if let Some(collided_ent) =
                                        map.creature_map.get(&map.get_index(next_x, next_y))
                                    {
                                        if let Some(fragile) = breakables.get_mut(*collided_ent) {
                                            fragile.was_hit = true;
                                            p_builder.make_hit_particle(*collided_ent);
                                        }

                                        stuns
                                            .insert(
                                                *collided_ent,
                                                crate::Stunned {
                                                    duration: crate::consts::WALL_HIT_STUN_DURATION,
                                                },
                                            )
                                            .ok();
                                    }

                                    if let Some(view) = viewables.get(ent_hit) {
                                        if map.visible_tiles[map.get_index(next_x, next_y)] {
                                            if ent_hit == *player {
                                                log.add("You are knocked into something");
                                            } else {
                                                log.add(format!(
                                                    "A {} is knocked into something",
                                                    view.name.to_lowercase()
                                                ));
                                            }
                                        }
                                    }

                                    next_x -= offset.x;
                                    next_y -= offset.y;

                                    stuns
                                        .insert(
                                            ent_hit,
                                            crate::Stunned {
                                                duration: crate::consts::WALL_HIT_STUN_DURATION,
                                            },
                                        )
                                        .ok();
                                    break;
                                }

                                if map.is_tile_water(next_x, next_y) {
                                    stuns
                                        .insert(
                                            ent_hit,
                                            crate::Stunned {
                                                duration: crate::consts::WALL_HIT_STUN_DURATION,
                                            },
                                        )
                                        .ok();

                                    if let Some(view) = viewables.get(ent_hit) {
                                        if map.visible_tiles[map.get_index(next_x, next_y)] {
                                            if ent_hit == *player {
                                                log.add("You are knocked into the water");
                                            } else {
                                                log.add(format!(
                                                    "A {} is knocked into the water",
                                                    view.name.to_lowercase()
                                                ));
                                            }
                                        }
                                    }

                                    break;
                                }
                            }

                            movements
                                .insert(
                                    ent_hit,
                                    crate::MoveIntent {
                                        loc: rltk::Point::new(next_x, next_y),
                                        force_facing: None,
                                        delay: 0,
                                    },
                                )
                                .ok();
                        }
                    }
                    crate::AttackTrait::Pull { amount, pass_over } => {
                        let ents_hit = self.get_hit_entities(&mut p_builder, &map, ent, intent);
                        for (ent_hit, _) in ents_hit {
                            let src_pos = positions.get(ent).unwrap().as_point();
                            let ent_pos = positions.get(ent_hit).unwrap().as_point();

                            let offset = crate::Direction::get_direction_towards(ent_pos, src_pos)
                                .unwrap()
                                .to_point();
                            let mut pull_path = rltk::line2d_bresenham(src_pos, ent_pos);
                            pull_path.pop();

                            // check for collision
                            let mut next_x = ent_pos.x;
                            let mut next_y = ent_pos.y;
                            for _ in 0..amount {
                                if let Some(path_pos) = pull_path.pop() {
                                    next_x = path_pos.x;
                                    next_y = path_pos.y;
                                } else {
                                    next_x += offset.x;
                                    next_y += offset.y;
                                }

                                // if we collide into something, rewind the attempted movement
                                // and insert a stun
                                if !map.is_tile_valid(next_x, next_y)
                                    || (!pass_over && next_x == src_pos.x && next_y == src_pos.y)
                                {
                                    if let Some(collided_ent) =
                                        map.creature_map.get(&map.get_index(next_x, next_y))
                                    {
                                        if let Some(fragile) = breakables.get_mut(*collided_ent) {
                                            fragile.was_hit = true;
                                            p_builder.make_hit_particle(*collided_ent);
                                        }

                                        stuns
                                            .insert(
                                                *collided_ent,
                                                crate::Stunned {
                                                    duration: crate::consts::WALL_HIT_STUN_DURATION,
                                                },
                                            )
                                            .ok();
                                        break;
                                    }

                                    next_x -= offset.x;
                                    next_y -= offset.y;

                                    stuns
                                        .insert(
                                            ent_hit,
                                            crate::Stunned {
                                                duration: crate::consts::WALL_HIT_STUN_DURATION,
                                            },
                                        )
                                        .ok();
                                    break;
                                }

                                if map.is_tile_water(next_x, next_y) {
                                    stuns
                                        .insert(
                                            ent_hit,
                                            crate::Stunned {
                                                duration: crate::consts::WALL_HIT_STUN_DURATION,
                                            },
                                        )
                                        .ok();

                                    if let Some(view) = viewables.get(ent_hit) {
                                        if map.visible_tiles[map.get_index(next_x, next_y)] {
                                            if ent_hit == *player {
                                                log.add("You are pulled into the water");
                                            } else {
                                                log.add(format!(
                                                    "A {} is pulled into the water",
                                                    view.name.to_lowercase()
                                                ));
                                            }
                                        }
                                    }

                                    break;
                                }
                            }

                            movements
                                .insert(
                                    ent_hit,
                                    crate::MoveIntent {
                                        loc: rltk::Point::new(next_x, next_y),
                                        force_facing: None,
                                        delay: 0,
                                    },
                                )
                                .ok();
                        }
                    }
                    crate::AttackTrait::Damage { amount } => {
                        let ents_hit = self.get_hit_entities(&mut p_builder, &map, ent, intent);
                        for (ent_hit, hit_locs) in ents_hit {
                            if invulns.get(ent_hit).is_some() {
                                continue;
                            }

                            if let Some(fragile) = breakables.get_mut(ent_hit) {
                                fragile.was_hit = true;
                                p_builder.make_hit_particle(ent_hit);
                            }

                            if let Some(aff_health) = healths.get_mut(ent_hit) {
                                aff_health.current -= amount;

                                let pos = positions.get(ent_hit).unwrap();
                                if let Some(view) = viewables.get(ent_hit) {
                                    if map.visible_tiles[map.get_index(pos.x, pos.y)] {
                                        if ent_hit == *player {
                                            log.add(format!("You are hit for {}", amount));
                                        } else {
                                            log.add(format!(
                                                "A {} is hit for {}",
                                                view.name.to_lowercase(),
                                                amount
                                            ));
                                        }
                                    }
                                }

                                if let Some(aff_part) = multis.get_mut(ent_hit) {
                                    for part in aff_part.part_list.iter_mut() {
                                        for part_pos in part.symbol_map.keys() {
                                            let adj_part_pos = pos.as_point() + *part_pos;

                                            if hit_locs.contains(&adj_part_pos) {
                                                part.health -= 1;
                                                break;
                                            }
                                        }
                                    }
                                }

                                p_builder.make_hit_particle(ent_hit);
                            }
                        }
                    }
                    crate::AttackTrait::Stun { duration } => {
                        let ents_hit = self.get_hit_entities(&mut p_builder, &map, ent, intent);
                        for (ent_hit, _) in ents_hit {
                            if let Some(fragile) = breakables.get_mut(ent_hit) {
                                fragile.was_hit = true;
                            }

                            stuns.insert(ent_hit, crate::Stunned { duration }).ok();
                            p_builder.make_stun_particle(ent_hit);
                        }
                    }
                    crate::AttackTrait::Movement { delay } => {
                        let targets = attack_type::each_attack_target(intent.main, intent.loc);
                        assert!(targets.len() == 1);

                        movements
                            .insert(
                                ent,
                                crate::MoveIntent {
                                    loc: targets[0],
                                    force_facing: None,
                                    delay,
                                },
                            )
                            .ok();
                    }
                    crate::AttackTrait::Invulnerable { duration } => {
                        invulns
                            .insert(ent, crate::Invulnerable { duration })
                            .expect("Failed to make player invulnerable");
                    }
                    crate::AttackTrait::NeedsStamina { amount } => {
                        if let Some(stamina) = stams.get_mut(ent) {
                            stamina.current -= amount;
                            stamina.recover = false;
                        }
                    }
                    crate::AttackTrait::Heal { amount: _ } => {
                        //
                    }
                    crate::AttackTrait::LanceCharge { dir } => {
                        if ent == *player {
                            *run_state = crate::RunState::Charging { dir, speed: 1 };
                        }
                    }
                    crate::AttackTrait::FollowsPath { step_delay, on_hit } => {
                        if let Some(pos) = positions.get(ent) {
                            let mut path =
                                rltk::line2d(rltk::LineAlg::Bresenham, intent.loc, pos.as_point());
                            path.pop();
                            path.reverse();

                            let projectile = entities.create();
                            attack_paths
                                .insert(
                                    projectile,
                                    crate::AttackPath {
                                        path,
                                        index: 0,
                                        step_delay,
                                        cur_delay: 0,
                                        on_hit,
                                    },
                                )
                                .ok();
                        }
                    }
                    crate::AttackTrait::CreatesWalls => {
                        let targets = attack_type::each_attack_target(intent.main, intent.loc);
                        for t in targets.iter() {
                            if !map.is_tile_occupied(t.x, t.y) {
                                s_builder.spawn(crate::SpawnRequest {
                                    position: *t,
                                    spawn_type: crate::SpawnType::Wall,
                                })
                            }
                        }
                    }
                }
            }
        }

        for done in finished_attacks.iter() {
            attacks.remove(*done);
            attacks_in_progress.remove(*done);
        }
    }
}

impl AttackSystem {
    fn get_hit_entities(
        &mut self,
        p_builder: &mut crate::ParticleBuilder,
        map: &crate::Map,
        ent: Entity,
        intent: &crate::AttackIntent,
    ) -> HashMap<specs::Entity, Vec<rltk::Point>> {
        let targets = attack_type::each_attack_target(intent.main, intent.loc);
        let mut ents_hit = HashMap::new();

        for point in targets {
            p_builder.make_bg_particle(point);
            let point_index = map.point2d_to_index(point);
            if let Some(aff_ent) = map.creature_map.get(&point_index) {
                // avoid self damage
                if *aff_ent == ent {
                    continue;
                }

                let hit_locs = ents_hit.entry(*aff_ent).or_insert(Vec::new());
                hit_locs.push(point);
            }
        }

        ents_hit
    }
}
