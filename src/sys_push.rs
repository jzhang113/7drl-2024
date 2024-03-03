use rltk::Algorithm2D;
use specs::prelude::*;

pub struct PushSystem;

const MAX_ITERS: i32 = 100;

impl<'a> System<'a> for PushSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::PushForce>,
        WriteStorage<'a, crate::Position>,
        ReadStorage<'a, crate::MultiTile>,
        WriteStorage<'a, crate::Viewshed>,
        WriteExpect<'a, crate::Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut pushes, mut positions, multitiles, mut viewsheds, mut map) = data;

        for (ent, force, pos, multis, view) in (
            &entities,
            &pushes,
            &mut positions,
            (&multitiles).maybe(),
            (&mut viewsheds).maybe(),
        )
            .join()
        {
            let final_dest = match multis {
                None => apply_forces(ent, force, pos, &mut *map),
                Some(multi) => apply_forces_to_multi(ent, force, pos, multi, &mut *map),
            };

            *pos = crate::Position {
                x: final_dest.x,
                y: final_dest.y,
            };

            if let Some(view) = view {
                view.dirty = true;
            }
        }

        pushes.clear();
    }
}

fn apply_forces(
    ent: Entity,
    force: &crate::PushForce,
    pos: &crate::Position,
    map: &mut crate::Map,
) -> rltk::Point {
    let mut prev_point;
    let mut new_point = pos.as_point();
    let mut applied_force = force.delta;

    for _ in 0..MAX_ITERS {
        prev_point = new_point;
        new_point = prev_point + applied_force;

        match calc_dest_force(ent, applied_force, prev_point, new_point, map) {
            None => return new_point,
            Some((updated_point, new_force)) => {
                applied_force = new_force;
                new_point = updated_point;
            }
        }
    }

    dbg!("can't find a place to move to");
    rltk::Point::zero()
}

fn calc_dest_force(
    _ent: Entity,
    applied_force: rltk::Point,
    prev_point: rltk::Point,
    new_point: rltk::Point,
    map: &mut crate::Map,
) -> Option<(rltk::Point, rltk::Point)> {
    let new_idx = map.get_index(new_point.x, new_point.y);

    // if we can fit in the new tile, we are done
    if !map.blocked_tiles[new_idx] {
        return None;
    }

    // TODO: assuming forces at most 1 tile (will also need to update collision checking sys_partmove to support higher forces)
    let refl_h_force = rltk::Point::new(-applied_force.x, applied_force.y);
    let refl_h_pos = new_point + refl_h_force;
    let refl_h_idx = map.get_index(refl_h_pos.x, refl_h_pos.y);
    if !map.blocked_tiles[refl_h_idx] {
        return Some((new_point, refl_h_force));
    }

    let refl_v_force = rltk::Point::new(applied_force.x, -applied_force.y);
    let refl_v_pos = new_point + refl_v_force;
    let refl_v_idx = map.get_index(refl_v_pos.x, refl_v_pos.y);
    if !map.blocked_tiles[refl_v_idx] {
        return Some((new_point, refl_v_force));
    }

    // Move to an open adjacent square
    let mut open_list = Vec::new();
    for dx in -1..=1 {
        for dy in -1..1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let new_x = prev_point.x + dx;
            let new_y = prev_point.y + dy;

            if map.in_bounds(rltk::Point::new(new_x, new_y)) {
                let new_map_idx = map.get_index(new_x, new_y);
                if !map.blocked_tiles[new_map_idx] {
                    open_list.push(rltk::Point::new(dx, dy));
                }
            }
        }
    }

    if !open_list.is_empty() {
        open_list.sort_by(|d1, d2| {
            let dist1 = rltk::DistanceAlg::Manhattan.distance2d(applied_force, *d1);
            let dist2 = rltk::DistanceAlg::Manhattan.distance2d(applied_force, *d2);
            dist1.partial_cmp(&dist2).unwrap()
        });

        return Some((prev_point, *open_list.first().unwrap()));
    }

    // nothing open nearby, check again at one of the reflected positions
    Some((prev_point, refl_h_force))
}

// TODO: Not even gonna think about this right now
fn apply_forces_to_multi(
    ent: Entity,
    force: &crate::PushForce,
    pos: &crate::Position,
    multi: &crate::MultiTile,
    map: &mut crate::Map,
) -> rltk::Point {
    pos.as_point()
}
