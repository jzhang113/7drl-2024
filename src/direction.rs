use derivative::Derivative;
use rltk::Point;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Derivative)]
#[derivative(Hash)]
pub enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    pub fn left(&self) -> Direction {
        match self {
            Direction::N => Direction::W,
            Direction::E => Direction::N,
            Direction::S => Direction::E,
            Direction::W => Direction::S,
        }
    }

    pub fn right(&self) -> Direction {
        match self {
            Direction::N => Direction::E,
            Direction::E => Direction::S,
            Direction::S => Direction::W,
            Direction::W => Direction::N,
        }
    }

    pub fn opp(&self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::E => Direction::W,
            Direction::S => Direction::N,
            Direction::W => Direction::E,
        }
    }

    pub fn to_point(&self) -> rltk::Point {
        match self {
            Direction::N => Point::new(0, -1),
            Direction::E => Point::new(1, 0),
            Direction::S => Point::new(0, 1),
            Direction::W => Point::new(-1, 0),
        }
    }

    pub fn get_direction_towards(from: Point, goal: Point) -> Option<Direction> {
        let dx = goal.x - from.x;
        let dy = goal.y - from.y;

        if dx.abs() > dy.abs() {
            match dx.signum() {
                1 => Some(crate::Direction::E),
                -1 => Some(crate::Direction::W),
                _ => unreachable!(), // if dx.signum is 0, dx = 0, but we can't be in this branch in that case
            }
        } else {
            match dy.signum() {
                1 => Some(crate::Direction::S),
                -1 => Some(crate::Direction::N),
                _ => None,
            }
        }
    }

    pub fn point_in_direction(from: Point, direction: Direction) -> Point {
        from + direction.to_point()
    }
}
