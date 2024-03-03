use std::collections::HashMap;

#[derive(Clone)]
pub struct MonsterPart {
    pub symbol_map: HashMap<rltk::Point, rltk::FontCharType>,
    pub health: i32,
    pub max_health: i32,
}

impl MonsterPart {
    pub fn get_bounds(&self) -> rltk::Rect {
        if self.symbol_map.is_empty() {
            return rltk::Rect::zero();
        }

        let first_pos = self.symbol_map.keys().next().unwrap();
        let mut min_x = first_pos.x;
        let mut min_y = first_pos.y;
        let mut max_x = first_pos.x;
        let mut max_y = first_pos.y;

        for point in self.symbol_map.keys() {
            min_x = std::cmp::min(min_x, point.x);
            min_y = std::cmp::min(min_y, point.y);
            max_x = std::cmp::max(max_x, point.x);
            max_y = std::cmp::max(max_y, point.y);
        }

        rltk::Rect::with_exact(min_x, min_y, max_x, max_y)
    }
}

pub fn all_bounds(part_list: &Vec<MonsterPart>) -> rltk::Rect {
    let mut bounds = rltk::Rect::zero();
    for part in part_list {
        let part_bounds = part.get_bounds();
        bounds.x1 = std::cmp::min(bounds.x1, part_bounds.x1);
        bounds.y1 = std::cmp::min(bounds.y1, part_bounds.y1);
        bounds.x2 = std::cmp::max(bounds.x2, part_bounds.x2);
        bounds.y2 = std::cmp::max(bounds.y2, part_bounds.y2);
    }

    bounds
}
