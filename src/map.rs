use rltk::{Algorithm2D, BaseMap, Point, Rect};
use specs::Entity;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    Water,
    DownStairs,
    NewLevel,
}

#[derive(Default, Clone)]
struct SearchArgs {
    search_entity: Option<Entity>,
    multi_component: Option<Vec<crate::MonsterPart>>,
}

#[derive(Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub name: String,
    pub camera: crate::Camera,
    pub color_map: Vec<rltk::RGB>,
    pub item_map: HashMap<usize, Entity>,
    pub creature_map: HashMap<usize, Entity>,
    pub known_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub blocked_vision: Vec<bool>,
    search_args: SearchArgs,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall || self.blocked_vision[idx]
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Manhattan.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    pub fn new(
        width: i32,
        height: i32,
        name: &String,
        map_color: &String,
        rng: &mut rltk::RandomNumberGenerator,
    ) -> Self {
        let dim = (width * height).try_into().unwrap();
        let base_color = if map_color != "#FFFFFF" {
            rltk::RGB::from_hex(map_color).unwrap().to_hsv()
        } else {
            rltk::HSV::from_f32(rng.rand::<f32>(), rng.rand::<f32>(), rng.rand::<f32>())
        };
        let color_map: Vec<rltk::RGB> = (0..dim)
            .map(|_| crate::map_wall_variant(base_color, rng))
            .collect();

        Self {
            tiles: vec![TileType::Wall; dim],
            rooms: vec![],
            width,
            height,
            name: name.clone(),
            camera: crate::Camera {
                origin: rltk::Point::zero(),
                map_width: width,
                map_height: height,
            },
            color_map: color_map,
            item_map: HashMap::new(),
            creature_map: HashMap::new(),
            known_tiles: vec![false; dim],
            visible_tiles: vec![false; dim],
            blocked_tiles: vec![false; dim],
            blocked_vision: vec![false; dim], // this is probably sparse?
            search_args: SearchArgs::default(),
        }
    }

    pub fn get_index(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }

    pub fn reset_vision(&mut self) {
        let dim = (self.width * self.height).try_into().unwrap();
        self.blocked_vision = vec![false; dim];
    }

    pub fn set_blocked_tiles(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            let is_blocked = *tile == TileType::Wall;
            self.blocked_tiles[index] = is_blocked;
        }
    }

    pub fn is_tile_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let index = self.get_index(x, y);

        // walls are always invalid
        if self.tiles[index] == TileType::Wall {
            return false;
        }

        // TODO: multi-tile bodies can still walk into players since player doesn't have a collision
        // non-blocked tiles are always valid
        if !self.blocked_tiles[index] {
            return true;
        }

        // blocked tiles can be valid if they belong to the search_entity (creatures are not blocked by themselves)
        let result = match self.search_args.search_entity {
            Some(search_entity) => match self.creature_map.get(&index) {
                Some(map_entity) => *map_entity == search_entity,
                None => false,
            },
            None => false,
        };

        result
    }

    pub fn is_tile_occupied(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let index = self.get_index(x, y);
        if self.blocked_tiles[index] {
            return true;
        }

        // walls are automatically set to blocked
        if self.is_tile_water(x, y) {
            return true;
        }

        false
    }

    pub fn is_tile_water(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let index = self.get_index(x, y);
        self.tiles[index] == TileType::Water
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.is_tile_valid(x, y) {
            return false;
        }

        // if search_entity is a multi-tile entity, test the actual move first
        if let Some(multitiles) = &self.search_args.multi_component {
            for part in multitiles {
                for part_pos in part.symbol_map.keys() {
                    let new_x = x + part_pos.x;
                    let new_y = y + part_pos.y;

                    if !self.is_tile_valid(new_x, new_y) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn set_additional_args(
        &mut self,
        entity: Entity,
        multi_component: Option<&crate::MultiTile>,
    ) {
        self.search_args.search_entity = Some(entity);
        self.search_args.multi_component = multi_component.map(|comp| comp.part_list.clone());
    }

    pub fn is_exit_valid_for(
        &mut self,
        x: i32,
        y: i32,
        entity: Entity,
        multi_component: Option<&crate::MultiTile>,
    ) -> bool {
        self.set_additional_args(entity, multi_component);
        self.is_exit_valid(x, y)
    }

    pub fn get_available_exits_for(
        &mut self,
        idx: usize,
        entity: Entity,
        multi_component: Option<&crate::MultiTile>,
    ) -> rltk::SmallVec<[(usize, f32); 10]> {
        self.set_additional_args(entity, multi_component);
        self.get_available_exits(idx)
    }

    pub(crate) fn build_room(&mut self, room: Rect) {
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let index = self.get_index(x, y);
                self.tiles[index] = TileType::Floor;
                self.color_map[index] = crate::map_floor_color();
            }
        }

        self.rooms.push(room);
    }

    /// Create a hallway of TileType::Floor between the given start and end points
    /// The hallway will always be built horizontally from the start position and vertically from the end position
    pub(crate) fn build_hallway(&mut self, start: Point, end: Point) {
        let xrange;
        let yrange;

        if start.x > end.x {
            xrange = (end.x - start.x)..=0;
        } else {
            xrange = 0..=(end.x - start.x);
        }

        if start.y > end.y {
            yrange = 0..=(start.y - end.y);
        } else {
            yrange = (start.y - end.y)..=0;
        }

        for dx in xrange {
            let next_x = start.x + dx;
            let next_y = start.y;

            let index = self.get_index(next_x, next_y);
            self.tiles[index] = TileType::Floor;
            self.color_map[index] = crate::map_floor_color();
        }

        for dy in yrange {
            let next_x = end.x;
            let next_y = end.y + dy;

            let index = self.get_index(next_x, next_y);
            self.tiles[index] = TileType::Floor;
            self.color_map[index] = crate::map_floor_color();
        }
    }

    pub fn track_item(&mut self, data: Entity, point: Point) -> bool {
        let index = self.point2d_to_index(point);

        if self.item_map.get(&index).is_some() {
            false
        } else {
            self.item_map.insert(index, data);
            true
        }
    }

    pub fn untrack_item(&mut self, point: Point) -> Option<Entity> {
        let index = self.point2d_to_index(point);
        self.item_map.remove(&index)
    }

    fn update_multi_component(
        &mut self,
        entity: Entity,
        multi_component: &crate::MultiTile,
        index: usize,
        is_blocked: bool,
    ) {
        let point = self.index_to_point2d(index);

        for part in &multi_component.part_list {
            for part_pos in part.symbol_map.keys() {
                let part_pos_index = self.point2d_to_index(*part_pos + point);
                self.blocked_tiles[part_pos_index] = is_blocked;

                if is_blocked {
                    self.creature_map.insert(part_pos_index, entity);
                } else {
                    self.creature_map.remove(&part_pos_index);
                }
            }
        }
    }

    pub fn track_creature(
        &mut self,
        data: Entity,
        index: usize,
        multi_component: Option<&crate::MultiTile>,
    ) -> bool {
        if self.creature_map.get(&index).is_some() {
            false
        } else {
            self.blocked_tiles[index] = true;
            self.creature_map.insert(index, data);

            if let Some(multi_component) = multi_component {
                self.update_multi_component(data, &multi_component, index, true);
            }

            true
        }
    }

    pub fn untrack_creature(
        &mut self,
        index: usize,
        multi_component: Option<&crate::MultiTile>,
    ) -> Option<Entity> {
        self.blocked_tiles[index] = false;
        let entity = self.creature_map.remove(&index);

        if let Some(entity) = entity {
            if let Some(multi_component) = multi_component {
                self.update_multi_component(entity, &multi_component, index, false);
            }
        }

        entity
    }

    // move a creature on the map, updating creature_map and blocked_tiles as needed
    // this does not update the position component
    // returns false if the move could not be completed
    pub fn move_creature(
        &mut self,
        creature: Entity,
        prev: Point,
        next: Point,
        multi_component: Option<&crate::MultiTile>,
    ) -> bool {
        let prev_index = self.point2d_to_index(prev);
        let next_index = self.point2d_to_index(next);

        // if the destination is blocked by something other than us, quit moving
        if !self.is_exit_valid_for(next.x, next.y, creature, multi_component) {
            return false;
        }

        self.creature_map.remove(&prev_index);
        self.blocked_tiles[prev_index] = false;
        if let Some(multi_component) = multi_component {
            self.update_multi_component(creature, multi_component, prev_index, false);
        }

        self.creature_map.insert(next_index, creature);
        self.blocked_tiles[next_index] = true;
        if let Some(multi_component) = multi_component {
            self.update_multi_component(creature, multi_component, next_index, true);
        }

        true
    }
}
