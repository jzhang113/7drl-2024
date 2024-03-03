use crate::*;
use rltk::Rect;

pub fn build_rogue_map(
    width: i32,
    height: i32,
    depth: i32,
    rng: &mut rltk::RandomNumberGenerator,
) -> Map {
    let mut map = Map::new(
        width,
        height,
        &"BSP".to_string(),
        &"#FFFFFF".to_string(),
        rng,
    );

    const MAX_ROOMS: i32 = 12;
    const MIN_ROOM_WIDTH: i32 = 20;
    const MAX_ROOM_WIDTH: i32 = 50;
    const MIN_ROOM_HEIGHT: i32 = 20;
    const MAX_ROOM_HEIGHT: i32 = 50;

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_ROOM_WIDTH, MAX_ROOM_WIDTH);
        let h = rng.range(MIN_ROOM_HEIGHT, MAX_ROOM_HEIGHT);
        let x = rng.range(1, map.width - w - 1);
        let y = rng.range(1, map.height - h - 1);

        let new_room = Rect::with_size(x, y, w, h);
        let mut quit = false;

        for other_rooms in map.rooms.iter() {
            if other_rooms.intersect(&new_room) {
                quit = true;
            }
        }

        if quit {
            continue;
        }

        map.build_room(new_room);
        if map.rooms.len() > 1 {
            let new_center = new_room.center();
            let prev_center = map.rooms[map.rooms.len() - 2].center();

            if rng.rand::<f32>() > 0.5 {
                map.build_hallway(prev_center, new_center);
            } else {
                map.build_hallway(new_center, prev_center);
            }
        }
    }

    map.set_blocked_tiles();

    let exit_room = map.rooms.len() - 1;
    let exit_x = rng.range(map.rooms[exit_room].x1, map.rooms[exit_room].x2);
    let exit_y = rng.range(map.rooms[exit_room].y1, map.rooms[exit_room].y2);
    // map.level_exit = map.get_index(exit_x, exit_y);
    // println!("{}", map.level_exit);

    map
}

pub fn build_level(ecs: &mut specs::World, width: i32, height: i32, depth: i32) -> Map {
    let map = {
        let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
        build_rogue_map(width, height, depth, &mut rng)
    };

    // we need to clone the list of rooms so that spawner can borrow the map
    let cloned_rooms = map.rooms.clone();
    // let mut spawner = spawner::Spawner::new(ecs, &mut map, width);

    for room in cloned_rooms.iter() {
        let quality = depth;
        let mut spawn_ary = Vec::new();
        spawn_ary.push(
            spawn::spawner::build_mook
                as for<'r> fn(&'r mut specs::World, rltk::Point) -> specs::Entity,
        );
        spawn_ary.push(spawn::spawner::build_archer);
        // spawner.build(
        //     &room,
        //     0 + quality / 2,
        //     2 + quality,
        //     vec![0.7, 0.3],
        //     spawn_ary,
        // );

        let mut builder_ary = Vec::new();
        builder_ary.push(spawn::spawner::build_empty_barrel);

        // spawner.build_with_quality(&room, 5, 10, depth, vec![1.0], builder_ary);
    }

    map
}
