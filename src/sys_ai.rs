use crate::{Map, MoveIntent};
use rltk::Algorithm2D;
use specs::prelude::*;

pub enum Behavior {
    Sleep,
    Wander,
    Chase { target_point: rltk::Point },
    Attack { info: AttackInfo },
    AttackStartup { turns_left: u32, info: AttackInfo },
    AttackRecovery { turns_left: u32, info: AttackInfo },
    Flee,
}

#[derive(Clone)]
pub enum NextIntent {
    None,
    Attack { intent: crate::AttackIntent },
    Move { intent: crate::MoveIntent },
    PartMove { intent: crate::PartMoveIntent },
}

#[derive(Copy, Clone)]
pub struct AttackInfo {
    attack_type: crate::AttackType,
    attack_loc: rltk::Point,
}

struct AiStepData<'a> {
    ent: Entity,
    pos: &'a crate::Position,
    state: &'a mut crate::AiState,
    viewshed: &'a crate::Viewshed,
    moveset: &'a crate::Moveset,
    multi: Option<&'a crate::MultiTile>,
    player_point: rltk::Point,
    map: &'a mut crate::Map,
    p_builder: &'a mut crate::ParticleBuilder,
    rng: &'a mut rltk::RandomNumberGenerator,
}

pub struct AiSystem;

impl<'a> System<'a> for AiSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::CanActFlag>,
        ReadStorage<'a, crate::Position>,
        WriteStorage<'a, crate::MoveIntent>,
        WriteStorage<'a, crate::AttackIntent>,
        WriteStorage<'a, crate::PartMoveIntent>,
        WriteStorage<'a, crate::AiState>,
        ReadStorage<'a, crate::Viewshed>,
        ReadStorage<'a, crate::Moveset>,
        ReadStorage<'a, crate::MultiTile>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, crate::Map>,
        WriteExpect<'a, crate::ParticleBuilder>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut can_act,
            positions,
            mut moves,
            mut attacks,
            mut part_moves,
            mut states,
            viewsheds,
            movesets,
            multis,
            player,
            mut map,
            mut p_builder,
            mut rng,
        ) = data;
        let mut turn_done = Vec::new();
        let player_point = positions.get(*player).unwrap().as_point();

        for (ent, _turn, pos, state, viewshed, moveset, multi) in (
            &entities,
            &can_act,
            &positions,
            &mut states,
            &viewsheds,
            &movesets,
            (&multis).maybe(),
        )
            .join()
        {
            let action = self.next_step(AiStepData {
                ent,
                pos,
                state,
                viewshed,
                moveset,
                multi,
                player_point,
                map: &mut *map,
                p_builder: &mut *p_builder,
                rng: &mut *rng,
            });

            match action {
                NextIntent::Attack { intent } => {
                    attacks
                        .insert(ent, intent)
                        .expect("Failed to insert attack from AI");
                }
                NextIntent::Move { intent } => {
                    moves
                        .insert(ent, intent)
                        .expect("Failed to insert movement from AI");
                }
                NextIntent::PartMove { intent } => {
                    part_moves
                        .insert(ent, intent)
                        .expect("Failed to insert part movement from AI");
                }
                NextIntent::None => {}
            }

            turn_done.push(ent);
        }

        for done in turn_done.iter() {
            can_act.remove(*done);
        }
    }
}

impl AiSystem {
    fn next_step(&mut self, data: AiStepData) -> NextIntent {
        loop {
            match data.state.status {
                Behavior::Sleep => {
                    // the do nothing state
                    // TODO: trigger wake up
                    return NextIntent::None;
                }
                Behavior::Wander => {
                    if Self::can_see_target(data.viewshed, data.player_point) {
                        data.state.status = Behavior::Chase {
                            target_point: data.player_point,
                        };
                    } else {
                        return Self::move_random(data);
                    }
                }
                Behavior::Chase { target_point } => {
                    if Self::can_see_target(data.viewshed, data.player_point) {
                        // track the player's current position
                        data.state.status = Behavior::Chase {
                            target_point: data.player_point,
                        };

                        // check if we have any attacks that can hit
                        let orig_point = data.pos.as_point();

                        let rolled_prob: f32 = data.rng.rand();
                        let mut cumul_prob: f32 = 0.0;
                        let mut attack_found = false;

                        // TODO: smarter attack selection
                        // this is fine when all of the attacks have similar attack ranges
                        // however, we might run into cases where we are in range to attack, but we decided to use an attack thats not valid
                        for (potential_attack, chance) in data.moveset.moves.iter() {
                            cumul_prob += chance;
                            if rolled_prob > cumul_prob {
                                continue;
                            }

                            if let Some(attack_loc) = crate::attack_type::is_attack_valid(
                                *potential_attack,
                                orig_point,
                                data.player_point,
                            ) {
                                data.state.status = Behavior::AttackStartup {
                                    turns_left: crate::attack_type::get_startup(*potential_attack),
                                    info: AttackInfo {
                                        attack_type: *potential_attack,
                                        attack_loc: attack_loc,
                                    },
                                };
                                attack_found = true;
                                break;
                            }
                        }

                        if !attack_found {
                            // if we can't hit, just move towards the player
                            return Self::move_towards(data.player_point, data);
                        }
                    } else {
                        // we don't see the player, move to the last tracked point
                        return Self::move_towards(target_point, data);
                    }
                }
                Behavior::AttackStartup { turns_left, info } => {
                    if turns_left > 0 {
                        data.state.status = Behavior::AttackStartup {
                            turns_left: turns_left - 1,
                            info,
                        };

                        return crate::get_startup_action(info.attack_type, turns_left as usize);
                    } else {
                        data.state.status = Behavior::Attack { info };
                    }
                }
                Behavior::Attack { info } => {
                    let intent = crate::attack_type::get_attack_intent(
                        info.attack_type,
                        info.attack_loc,
                        None,
                    );

                    data.state.status = Behavior::AttackRecovery {
                        turns_left: crate::attack_type::get_recovery(info.attack_type),
                        info,
                    };

                    return NextIntent::Attack { intent };
                }
                Behavior::AttackRecovery { turns_left, info } => {
                    if turns_left > 0 {
                        data.state.status = Behavior::AttackRecovery {
                            turns_left: turns_left - 1,
                            info,
                        };

                        return crate::get_recovery_action(info.attack_type, turns_left as usize);
                    } else {
                        if Self::can_see_target(data.viewshed, data.player_point) {
                            data.state.status = Behavior::Chase {
                                target_point: data.player_point,
                            };
                        } else {
                            data.state.status = Behavior::Wander;
                        }
                    }
                }
                Behavior::Flee => {
                    // TODO
                    return NextIntent::None;
                }
            }
        }
    }

    fn move_random(data: AiStepData) -> NextIntent {
        let curr_index = data.map.get_index(data.pos.x, data.pos.y);

        // pick a random tile we can move to
        let exits = data
            .map
            .get_available_exits_for(curr_index, data.ent, data.multi);
        if exits.len() > 0 {
            let exit_index = data.rng.range(0, exits.len());
            let chosen_exit = exits[exit_index].0;
            return NextIntent::Move {
                intent: MoveIntent {
                    loc: data.map.index_to_point2d(chosen_exit),
                    force_facing: None,
                },
            };
        } else {
            // TODO: help we're stuck
            return NextIntent::None;
        }
    }

    fn move_towards(target_point: rltk::Point, data: AiStepData) -> NextIntent {
        let curr_index = data.map.get_index(data.pos.x, data.pos.y);
        let target_index = data.map.point2d_to_index(target_point);
        let path = Self::get_path_to(data.ent, data.map, curr_index, target_index, data.multi);

        match path {
            None => {
                if let Some(path) = &data.state.prev_path {
                    if data.state.path_step < path.steps.len() {
                        let next_pos = data.map.index_to_point2d(path.steps[data.state.path_step]);
                        data.state.path_step += 1;

                        return NextIntent::Move {
                            intent: MoveIntent {
                                loc: next_pos,
                                force_facing: None,
                            },
                        };
                    }
                }

                // no path to target, attempt to move towards the target
                let curr_point = data.pos.as_point();
                let dir = crate::Direction::get_direction_towards(curr_point, target_point)
                    .unwrap_or(crate::Direction::N);
                let next_point = crate::Direction::point_in_direction(curr_point, dir);

                if data
                    .map
                    .is_exit_valid_for(next_point.x, next_point.y, data.ent, data.multi)
                {
                    return NextIntent::Move {
                        intent: MoveIntent {
                            loc: next_point,
                            force_facing: None,
                        },
                    };
                }

                // can't move towards the target, just make a random move
                return Self::move_random(data);
            }
            Some(path) => {
                let next_pos = data.map.index_to_point2d(path.steps[1]);
                data.state.prev_path = Some(path);
                data.state.path_step = 2;

                return NextIntent::Move {
                    intent: MoveIntent {
                        loc: next_pos,
                        force_facing: None,
                    },
                };
            }
        }
    }

    fn can_see_target(viewshed: &crate::Viewshed, target: rltk::Point) -> bool {
        viewshed
            .visible
            .iter()
            .any(|pos| pos.x == target.x && pos.y == target.y)
    }

    fn get_path_to(
        entity: Entity,
        map: &mut Map,
        curr_index: usize,
        target_index: usize,
        multi_component: Option<&crate::MultiTile>,
    ) -> Option<rltk::NavigationPath> {
        map.set_additional_args(entity, multi_component);
        let path = rltk::a_star_search(curr_index, target_index, &*map);

        if path.success && path.steps.len() > 1 {
            return Some(path);
        } else {
            println!("No path exists!");
            return None;
        }
    }
}
