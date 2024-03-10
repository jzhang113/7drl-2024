use super::{Heal, Health, Map, Position, RunState, Schedulable};
use rltk::Algorithm2D;
use specs::prelude::*;

pub struct PickupSystem;

impl<'a> System<'a> for PickupSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RunState>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Schedulable>,
        ReadStorage<'a, Heal>,
        ReadStorage<'a, crate::EarthScroll>,
        WriteStorage<'a, Health>,
        WriteExpect<'a, crate::GameLog>,
        WriteExpect<'a, crate::Spawner>,
        WriteExpect<'a, rltk::RandomNumberGenerator>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player,
            mut map,
            run_state,
            positions,
            scheds,
            heals,
            earth_scrolls,
            mut healths,
            mut log,
            mut s_builder,
            mut rng
        ) = data;
        let mut consumed = Vec::new();

        for (ent, pos, health, _) in (&entities, &positions, &mut healths, &scheds).join() {
            if ent != *player {
                continue;
            }

            let index = map.get_index(pos.x, pos.y);

            match map.item_map.get(&index) {
                None => {}
                Some(item_ent) => {
                    if let Some(healing) = heals.get(*item_ent) {
                        let amt = std::cmp::min(healing.amount as i32, health.max - health.current);
                        if amt == 0 {
                            log.add("You have no need of healing right now");
                        } else {
                            health.current += amt;
                            let ie = map.untrack_item(rltk::Point::new(pos.x, pos.y)).unwrap();
                            consumed.push(ie);
                            log.add(format!("You pick up a potion and heal for {}", amt));
                        }
                    } else if let Some(es) = earth_scrolls.get(*item_ent) {
                        let tls = crate::range_type::resolve_range_at(
                            &crate::RangeType::Square { size: es.radius },
                            rltk::Point::new(pos.x, pos.y),
                        );

                        for tl in tls {
                            if !map.in_bounds(tl) {
                                continue;
                            }

                            if map.is_tile_occupied(tl.x, tl.y) {
                                continue;
                            }


                            if rng.rand::<f32>() < es.active_prob {
                                s_builder.spawn(crate::SpawnRequest {
                                    position: tl,
                                    spawn_type: crate::SpawnType::Wall,
                                });
                            }
                        }

                        let ie = map.untrack_item(rltk::Point::new(pos.x, pos.y)).unwrap();
                        consumed.push(ie);
                        log.add("You read a scroll of earth and several pillars appear");
                    }
                }
            }
        }

        for item in consumed {
            entities
                .delete(item)
                .expect("Failed to remove consumed item");
        }
    }
}
