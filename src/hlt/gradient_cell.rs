use hlt::position::Position;

pub struct GradientCell {
    pub position: Position,
    pub nearest_dropoff: Position,
    pub distance_to_dropoff: usize,
    pub value: f64,
    pub collection_amt: f64,
    pub surrounding_average: f64,
    pub move_cost: f64,
    pub my_occupy: bool,
    pub nearby_ship_count: i8,
    pub cells_effecting: i64,
    pub local_maxim: bool
}