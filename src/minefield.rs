use std::collections::{VecDeque, HashSet};
use notan::math::rand::*;

pub const BEGINNER: Params = Params {
    width: 8,
    height: 8,
    mines: 10,
};
pub const INTERMEDIATE: Params = Params {
    width: 16,
    height: 16,
    mines: 40,
};
pub const EXPERT: Params = Params {
    width: 30,
    height: 16,
    mines: 99,
};

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub width: usize,
    pub height: usize,
    pub mines: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mark {
    /// Mine flag, indicates 100% player certainty of a mine,
    /// and disables uncovering the marked field, for safety.
    Flag,

    /// "Danger, probably" marker, for fields that are sorta suspicious,
    /// but not yet worthy of The [Mark::Flag].
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
    covered_count: usize,
    params: Params,
    placed: bool,
    defeat: bool,
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

    pub fn uncover(&mut self) {
        self.cover = Cover::Down;
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

    fn is_hint(&self) -> bool {
        matches!(self.object, Object::Hint(_))
    }
}

impl Board {
    pub fn new(params: Params) -> Self {
        let size = params.width * params.height;
        Self {
            tiles: vec![Tile::new(); size],
            covered_count: size,
            placed: false,
            defeat: false,
            params,
        }
    }

    pub fn beginner() -> Self {
        Self::new(BEGINNER)
    }

    pub fn intermediate() -> Self {
        Self::new(INTERMEDIATE)
    }

    pub fn expert() -> Self {
        Self::new(EXPERT)
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.params.width, self.params.height)
    }

    pub fn tile(&self, x: usize, y: usize) -> Tile {
        let index = Self::coords_to_index(&self.params, x, y);
        self.tiles[index]
    }

    pub fn is_victory(&self) -> bool {
        !self.defeat && self.covered_count == self.params.mines
    }

    pub fn is_defeat(&self) -> bool {
        self.defeat
    }

    /// Primary interface for acting on a minefield.
    ///
    /// Corresponds to the action of uncovering a covered tile and either:
    ///
    /// - uncovering a hint
    /// - uncovering a blank, which triggers a flood-uncover
    /// - uncovering a mine, resulting in a game-over
    ///
    /// Uncovering every non-mine tile is the win condition.
    /// Note that the mine tiles are **not** required to be flagged (looking at you, speedrunners).
    pub fn handle_uncover(&mut self, x: usize, y: usize) {
        let tile_idx = Self::coords_to_index(&self.params, x, y);
        match self.tiles[tile_idx].cover {
            Cover::Up(_) => {
                self.tiles[tile_idx].uncover();
                if self.covered_count > self.params.mines {
                    self.covered_count -= 1;
                }
                match self.tiles[tile_idx].object {
                    Object::Mine => self.defeat = true,
                    Object::Blank => self.flood_uncover(x, y),
                    _ => ()
                }
            }
            Cover::Down => () //OPTIONAL: on uncover additional action when clicking hint
        }
    }

    /// Primary interface for acting on a minefield.
    ///
    /// Corresponds to the action of cycling through
    /// available covered-field marks (the [Mark] type).
    pub fn handle_mark(&mut self, x: usize, y: usize) {
        let tile_idx = Self::coords_to_index(&self.params, x, y);
        if let Cover::Up(mark) = self.tiles[tile_idx].cover {
            match mark {
                Mark::Flag => self.tiles[tile_idx].cover = Cover::Up(Mark::Unsure),
                Mark::Unsure => self.tiles[tile_idx].cover = Cover::Up(Mark::None),
                Mark::None => self.tiles[tile_idx].cover = Cover::Up(Mark::Flag),
            }
        }
    }

    /// A flood-fill-style uncovering procedure,
    /// where the uncovering "spills" over a surrounding area
    /// bounded by hint tiles (inclusive).
    ///
    /// Algorithmically, this is equivalent to a DFS/BFS traversal
    /// starting from a player-uncovered tile
    /// and stopping on already uncovered tiles and hint tiles.
    fn flood_uncover(&mut self, x: usize, y: usize) {
        let all_tiles = &mut self.tiles;
        let mut tile_pos = VecDeque::new();
        tile_pos.push_back((x, y));

        while let Some(tile) = tile_pos.pop_front() {
            let t_idx = Self::coords_to_index(&self.params, tile.0, tile.1);
            if all_tiles[t_idx].is_uncoverable() && !all_tiles[t_idx].is_mine() {
                all_tiles[t_idx].uncover();
                if self.covered_count > self.params.mines {
                    self.covered_count -= 1;
                }

                if !all_tiles[t_idx].is_hint() {
                    let min_x = if tile.0 > 0 { tile.0 - 1 } else { 0 };
                    let max_x = if tile.0 < self.params.width { tile.0 + 1 } else { tile.0 };
                    let min_y = if tile.1 > 0 { tile.1 - 1 } else { 0 };
                    let max_y = if tile.1 < self.params.height { tile.1 + 1 } else { tile.1 };

                    for i in min_x..=max_x {
                        for j in min_y..=max_y {
                            let idx = Self::coords_to_index(&self.params, i, j);
                            if all_tiles[idx].is_uncoverable() {
                                tile_pos.push_back((i, j));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get single dimension index of 2D tile
    ///
    /// This method exists purely because borrow checker took me hostage :)
    fn coords_to_index(params: &Params, x: usize, y: usize) -> usize {
        y * params.width + x
    }

    /// Place mines on the field.
    ///
    /// The `skip` argument contains board indices
    /// that shall not have a mine placed in.
    fn place_mines(&mut self, skip: &[usize]) {
        // i would put (usize, usize) here, since its just one point user clicks on + eventual flood
        // and then have this method be called in handle_uncover at the beginning
        if self.placed {
            return;
        }
        self.placed = true;
        let mut set = HashSet::new();
        let mut rng = thread_rng();

        for i in skip {
            set.insert(*i);
        }

        while set.len() < self.params.mines {
            let mine_idx: usize = rng.gen::<usize>() % self.tiles.len();
            if !set.contains(&mine_idx) {

            }
        }

        while let Some(index) = set.iter().next() {
            self.tiles[*index].object = Object::Mine;
        }

        self.place_hints();
    }

    fn place_hints(&mut self) {
        //TODO: Place hints here
    }
}
