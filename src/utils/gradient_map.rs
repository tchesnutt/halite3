use hlt::gradient_cell::GradientCell;

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GradientCell>>,
}

impl GradientMap {
   pub fn initialize(&self, gm: GameMap) -> GradientMap {
       let mut cells: Vec<Vec<GradientCell>> = Vec::with_capacity(gm.height);

       // do gradient work
   }
}