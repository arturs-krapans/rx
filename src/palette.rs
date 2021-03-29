use rgx::kit::Rgba8;

pub struct Palette {
    // TODO: Make this an `ArrayVec<[Rgba8; 256]>`.
    pub colors: Vec<Rgba8>,
    pub hover_color: Option<Rgba8>,
    pub cellsize: f32,
    pub x: f32,
    pub y: f32,
}

impl Palette {
    pub fn new(cellsize: f32) -> Self {
        Self {
            colors: Vec::with_capacity(256),
            hover_color: None,
            cellsize,
            x: 0.,
            y: 0.,
        }
    }

    pub fn add(&mut self, color: Rgba8) {
        // TODO: Ensure there are no duplicate colors.
        self.colors.push(color);
    }

    pub fn size(&self) -> usize {
        self.colors.len()
    }

    pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
        let mut x = x as i32 - self.x as i32;
        let mut y = y as i32 - self.y as i32;
        let cellsize = self.cellsize as i32;
        let size = self.size() as i32;

        let width = if size > 16 { cellsize * 2 } else { cellsize };
        let height = i32::min(size, 16) * cellsize;

        if x >= width || y >= height || x < 0 || y < 0 {
            self.hover_color = None;
            return;
        }

        x /= cellsize;
        y /= cellsize;

        let index = y + x * 16;

        self.hover_color = if index < size {
            Some(self.colors[index as usize])
        } else {
            None
        };
    }
}
