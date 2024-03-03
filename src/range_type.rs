use rltk::Point;

#[derive(PartialEq)]
pub enum RangeType {
    Empty,
    Single,
    Square { size: i32 },
    Diamond { size: i32 },
    Path { dest: rltk::Point },
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
        RangeType::Diamond { size } => {
            for dx in -size..=*size {
                for dy in -size..=*size {
                    if !(dx == 0 && dy == 0) && dx.abs() + dy.abs() <= *size {
                        targets.push(Point::new(center.x + dx, center.y + dy));
                    }
                }
            }
        }
        RangeType::Path { dest } => {
            targets = rltk::Bresenham::new(*dest, center).collect();
        }
        RangeType::Custom { offsets } => {
            for (dx, dy) in offsets {
                targets.push(center + Point::new(*dx, *dy))
            }
        }
    }

    targets
}
