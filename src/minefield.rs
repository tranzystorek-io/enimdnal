const EASY: Difficulty = Difficulty {
    width: 8,
    height: 8,
    mines: 10,
};
const MEDIUM: Difficulty = Difficulty {
    width: 16,
    height: 16,
    mines: 40,
};
const HARD: Difficulty = Difficulty {
    width: 30,
    height: 16,
    mines: 99,
};

pub struct Difficulty {
    width: usize,
    height: usize,
    mines: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mark {
    Flag,
    Unsure,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum Cover {
    Up(Mark),
    Down,
}

#[derive(Debug, Clone, Copy)]
pub enum Object {
    Mine,
    Hint(u8),
    Blank,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    cover: Cover,
    object: Object,
}

#[derive(Debug)]
pub struct Board {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
}

impl Mark {
    fn cycle(&mut self) {
        *self = match self {
            Self::None => Self::Flag,
            Self::Flag => Self::Unsure,
            Self::Unsure => Self::None,
        };
    }
}

impl Tile {
    fn new() -> Self {
        Self {
            cover: Cover::Up(Mark::None),
            object: Object::Blank,
        }
    }

    pub fn cover(&self) -> Cover {
        self.cover
    }

    pub fn object(&self) -> Object {
        self.object
    }

    fn is_uncoverable(&self) -> bool {
        matches!(self.cover, Cover::Up(mark) if mark != Mark::Flag)
    }

    fn is_mine(&self) -> bool {
        matches!(self.object, Object::Mine)
    }
}

impl Board {
    fn new(width: usize, height: usize, mines: usize) -> Self {
        let size = width * height;
        Self {
            tiles: vec![Tile::new(); size],
            width,
            height,
        }
    }

    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        let Difficulty {
            width,
            height,
            mines,
        } = difficulty;
        Self::new(width, height, mines)
    }

    pub fn easy() -> Self {
        Self::from_difficulty(EASY)
    }

    pub fn medium() -> Self {
        Self::from_difficulty(MEDIUM)
    }

    pub fn hard() -> Self {
        Self::from_difficulty(HARD)
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn tile(&self, x: usize, y: usize) -> Tile {
        let index = self.coords_to_index(x, y);
        self.tiles[index]
    }

    pub fn handle_uncover(&mut self, x: usize, y: usize) {
        todo!()
    }

    pub fn handle_mark(&mut self, x: usize, y: usize) {
        todo!()
    }

    fn flood_uncover(&mut self, x: usize, y: usize) {
        todo!()
    }

    fn coords_to_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

fn place_mines(tiles: &mut [Tile]) {
    todo!()
}

fn place_hints(tiles: &mut [Tile]) {
    todo!()
}
