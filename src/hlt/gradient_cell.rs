use hlt::position::Position;

pub struct GradientCell {
    pub position: Position,
    pub value: f64,
    pub collection_amt: f64,
    pub surrounding_average: f64,
    pub move_cost: f64,
    pub my_occupy: bool,
    pub nearby_ship_count: i8,
}