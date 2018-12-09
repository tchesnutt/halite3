use hlt::gradient_cell::GradientCell;

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GradientCell>>,
}

impl GradientMap {
    pub fn initialize(&self, gm: GameMap) -> GradientMap {
        //dup gamemap with GradienCells
        let height = gm.height;
        let width = gm.width;

        let mut cells: Vec<Vec<GradientCell>> = Vec::with_capacity(height);
        for y in 0..cells.len() {
            let mut row: Vec<GradientCell> = Vec::with_capacity(width);
            for x in 0..row.len() {
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                let value: i32 = gm.at_position(position).halite;
                // do gradient work

                let cell = GradientCell { position, value };
                row.push(cell);
            }
        }

        GradientMap {
            width,
            height,
            cells,
        }
    }

    pub fn adjust(&self) -> GradientCell {}
}
