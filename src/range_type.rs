use rltk::Point;

#[derive(PartialEq)]
pub enum RangeType {
    Empty,
    Single,
    Square { size: i32 },
    SquareInclusive { size: i32 },
    Ring { size: i32 },
    Diamond { size: i32 },
    Cross { size: i32 },
    Path { dest: Point },
    Ray { dir: crate::Direction, len: i32 },
    Custom { offsets: Vec<(i32, i32)> },
}

pub fn resolve_range_at(range: &RangeType, center: Point) -> Vec<Point> {
    let mut targets = Vec::new();

    match range {
        RangeType::Empty => {}
        RangeType::Single => {
            targets.push(center);
        }
        RangeType::Square { size } => {
            for x in center.x - size..=center.x + size {
                for y in center.y - size..=center.y + size {
                    if !(x == center.x && y == center.y) {
                        targets.push(Point::new(x, y));
                    }
                }
            }
        }
        RangeType::SquareInclusive { size } => {
            for x in center.x - size..=center.x + size {
                for y in center.y - size..=center.y + size {
                    targets.push(Point::new(x, y));
                }
            }
        }
        RangeType::Ring { size } => {
            for x in center.x - size..=center.x + size {
                targets.push(Point::new(x, center.y - size));
                targets.push(Point::new(x, center.y + size));
            }

            for y in center.y - size + 1..center.y + size {
                targets.push(Point::new(center.x - size, y));
                targets.push(Point::new(center.x + size, y));
            }
        }
        RangeType::Diamond { size } => {
            for dx in -size..=*size {
                for dy in -size..=*size {
                    if !(dx == 0 && dy == 0) && dx.abs() + dy.abs() <= *size {
                        targets.push(Point::new(center.x + dx, center.y + dy));
                    }
                }
            }
        }
        RangeType::Cross { size } => {
            for x in center.x - size..=center.x + size {
                if x != center.x {
                    targets.push(Point::new(x, center.y));
                }
            }

            for y in center.y - size..=center.y + size {
                if y != center.y {
                    targets.push(Point::new(center.x, y));
                }
            }
        }
        RangeType::Path { dest } => {
            targets = rltk::Bresenham::new(*dest, center).collect();
        }
        RangeType::Ray { dir, len } => {
            let mut cur_point = center;
            let offset = dir.to_point();

            for _ in 0..*len {
                cur_point = Point::new(cur_point.x + offset.x, cur_point.y + offset.y);
                targets.push(cur_point);
            }
        }
        RangeType::Custom { offsets } => {
            for (dx, dy) in offsets {
                targets.push(center + Point::new(*dx, *dy))
            }
        }
    }

    targets
}
