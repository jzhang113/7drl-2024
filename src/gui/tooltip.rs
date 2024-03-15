use super::consts::*;
use crate::*;
use std::collections::HashSet;

// TODO
pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let renderables = ecs.read_storage::<Renderable>();
    let viewables = ecs.read_storage::<Viewable>();
    let positions = ecs.read_storage::<Position>();
    let schedulables = ecs.read_storage::<Schedulable>();

    let mouse_point = ctx.mouse_point();
    let adjusted_point = mouse_point - rltk::Point::new(SIDE_W + 1, 1) + map.camera.origin;

    let mut tooltip = HashSet::new();

    for (_rend, view, pos) in (&renderables, &viewables, &positions).join() {
        if pos.as_point() == adjusted_point {
            tooltip.insert(view.name.to_string());
        }
    }

    if map.in_bounds(adjusted_point) {
        let ent = map
            .creature_map
            .get(&map.get_index(adjusted_point.x, adjusted_point.y));

        if let Some(ent) = ent {
            let vv = viewables.get(*ent).unwrap();

            tooltip.insert(vv.name.to_string());

            let ss = schedulables.get(*ent).unwrap();
            tooltip.insert(ss.current.to_string());
        }
    }

    if !tooltip.is_empty() {
        // placeholder
        ctx.print_color(
            1,
            1,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::GREY),
            tooltip.into_iter().collect::<Vec<_>>().concat(),
        );
    }
}
