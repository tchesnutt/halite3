use hlt::direction::Direction;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn directional_offset(&self, d: Direction) -> Position {
        let (dx, dy) = match d {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::Still => (0, 0),
        };

        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn get_surrounding_cardinals(&self) -> Vec<Position> {
        vec![
            self.directional_offset(Direction::North),
            self.directional_offset(Direction::South),
            self.directional_offset(Direction::East),
            self.directional_offset(Direction::West),
            self.directional_offset(Direction::Still),
        ]
    }

    pub fn same_position(&self, other_position: &Position) -> bool {
        if self.x == other_position.x && self.y == other_position.y {
            return true;
        };
        false
    }

    pub fn distance_to(&self, other_position: &Position, width: &usize, height: &usize) -> usize {
        let mut distance: usize = 0;
        let dx = (self.x - other_position.x).abs() as usize;
        let dy = (self.y - other_position.y).abs() as usize;

        let wrapped_dx = width - dx;
        let wrapped_dy = height - dy;

        distance += if wrapped_dx > dx { dx } else { wrapped_dx };
        distance += if wrapped_dy > dx { dy } else { wrapped_dy };

        distance
    }
}
