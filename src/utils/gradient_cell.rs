use hlt::position::Position;

pub struct GradientCell {
    pub position: Position,
    pub value: usize,
    pub my_occupy: bool,
    pub nearby_ship_count: i8,
}


