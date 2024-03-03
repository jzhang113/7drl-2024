use rltk::Point;
use std::cmp::{max, min};

pub const VIEW_W: i32 = 79;
pub const VIEW_H: i32 = 50;
pub const MAP_W: i32 = 120;
pub const MAP_H: i32 = 120;

#[derive(Copy, Clone)]
pub struct Camera {
    pub origin: Point,
    pub map_width: i32,
    pub map_height: i32,
}

impl Camera {
    pub fn update(&mut self, center: Point) {
        let origin_x = if self.map_width < VIEW_W {
            (self.map_width - VIEW_W) / 2
        } else {
            let top_left_x = max(center.x - VIEW_W / 2, 0);
            let max_x = max(self.map_width - VIEW_W, 0);
            min(top_left_x, max_x)
        };

        let origin_y = if self.map_height < VIEW_H {
            (self.map_height - VIEW_H) / 2
        } else {
            let top_left_y = max(center.y - VIEW_H / 2, 0);
            let max_y = max(self.map_height - VIEW_H, 0);
            min(top_left_y, max_y)
        };

        self.origin = Point::new(origin_x, origin_y);
    }

    pub fn on_screen(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x < self.origin.x + VIEW_W
            && point.y < self.origin.y + VIEW_H
    }

    pub fn iter(&self) -> CameraIterator {
        CameraIterator::new(
            max(self.origin.x, 0),
            max(self.origin.y, 0),
            self.map_width,
            self.map_height,
        )
    }
}

pub struct CameraIterator {
    initial: Point,
    x: i32,
    y: i32,
    map_width: i32,
    map_height: i32,
}

impl CameraIterator {
    fn new(initial_x: i32, initial_y: i32, map_width: i32, map_height: i32) -> Self {
        Self {
            initial: rltk::Point::new(initial_x, initial_y),
            x: initial_x - 1,
            y: initial_y,
            map_width,
            map_height,
        }
    }

    fn get_index(&self) -> usize {
        ((self.y * self.map_width) + self.x) as usize
    }
}

impl Iterator for CameraIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.x += 1;
        if self.x < min(self.map_width, self.initial.x + VIEW_W) {
            return Some(self.get_index());
        }

        self.x = self.initial.x;
        self.y += 1;
        if self.y < min(self.map_height, self.initial.y + VIEW_H) {
            return Some(self.get_index());
        }

        None
    }
}
