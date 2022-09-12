use notan::math::rand::seq::IteratorRandom;
use notan::math::rand::*;
use std::cmp::min;
use std::collections::{HashSet, VecDeque};
use std::ops::Range;

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
    covered: usize,
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
            covered: size,
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
        let index = self.coords_to_index(x, y);
        self.tiles[index]
    }

    pub fn is_victory(&self) -> bool {
        self.covered == self.params.mines
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
        let tile_idx = self.coords_to_index(x, y);

        if !self.placed {
            self.placed = true;
            self.place_mines(&[tile_idx]);
            self.place_hints();
        }

        match self.tiles[tile_idx].cover {
            Cover::Up(mark) => {
                if matches!(mark, Mark::Flag) {
                    return;
                }
                self.tiles[tile_idx].cover = Cover::Down;
                if self.covered > self.params.mines {
                    self.covered -= 1;
                }
                match self.tiles[tile_idx].object {
                    Object::Mine => self.defeat = true,
                    Object::Blank => self.flood_uncover(x, y),
                    _ => (),
                }
            }
            Cover::Down => (), //OPTIONAL: on uncover additional action when clicking hint
        }
    }

    /// Primary interface for acting on a minefield.
    ///
    /// Corresponds to the action of cycling through
    /// available covered-field marks (the [Mark] type).
    pub fn handle_mark(&mut self, x: usize, y: usize) {
        let tile_idx = self.coords_to_index(x, y);
        if let Cover::Up(mark) = &mut self.tiles[tile_idx].cover {
            mark.cycle();
        }
    }

    fn neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        let offsets = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        let width = self.params.width;
        let height = self.params.height;

        offsets.into_iter().filter_map(move |(off_x, off_y)| {
            let new_x = (x as i32 + off_x);
            let new_y = (y as i32 + off_y);
            let new_x_inbounds = new_x >= 0 && new_x < width as i32;
            let new_y_inbounds = new_y >= 0 && new_y < height as i32;
            if new_x_inbounds && new_y_inbounds {
                return Some((new_x as _, new_y as _));
            }
            None
        })
    }

    /// A flood-fill-style uncovering procedure,
    /// where the uncovering "spills" over a surrounding area
    /// bounded by hint tiles (inclusive).
    ///
    /// Algorithmically, this is equivalent to a DFS/BFS traversal
    /// starting from a player-uncovered tile
    /// and stopping on already uncovered tiles and hint tiles.
    fn flood_uncover(&mut self, x: usize, y: usize) {
        let mut tile_pos = VecDeque::new();
        let mut visited = HashSet::new();
        tile_pos.push_back((x, y));

        while let Some(tile) = tile_pos.pop_front() {
            if !visited.insert(tile) {
                continue;
            }
            let t_idx = self.coords_to_index(tile.0, tile.1);
            if self.tiles[t_idx].is_uncoverable() && !self.tiles[t_idx].is_mine() {
                self.tiles[t_idx].cover = Cover::Down;
                if self.covered > self.params.mines {
                    self.covered -= 1;
                }

                if !self.tiles[t_idx].is_hint() {
                    for (xx, yy) in self.neighbours(tile.0, tile.1) {
                        if self.tiles[self.coords_to_index(xx, yy)].is_uncoverable() {
                            tile_pos.push_back((xx, yy));
                        }
                    }
                }
            }
        }
    }

    /// Get single dimension index of 2D tile in tiles array
    fn coords_to_index(&self, x: usize, y: usize) -> usize {
        y * self.params.width + x
    }

    /// Place mines on the field.
    ///
    /// The `skip` argument contains board indices
    /// that shall not have a mine placed in.
    fn place_mines(&mut self, skip: &[usize]) {
        // i would put (usize, usize) here, since its just one point user clicks on + eventual flood
        // and then have this method be called in handle_uncover at the beginning
        let mut rng = thread_rng();
        let idx_range = Range {
            start: 0,
            end: self.tiles.len(),
        };

        let mines = idx_range
            .filter(|i| !skip.contains(i))
            .choose_multiple(&mut rng, self.params.mines);

        for mine in &mines {
            self.tiles[*mine].object = Object::Mine;
        }
    }

    fn place_hints(&mut self) {
        for x in 0..self.params.width {
            for y in 0..self.params.height {
                let idx = self.coords_to_index(x, y);
                if self.tiles[idx].is_mine() {
                    continue;
                }
                let mine_count = self
                    .neighbours(x, y)
                    .filter(|pos| self.tiles[self.coords_to_index(pos.0, pos.1)].is_mine())
                    .count();
                if mine_count > 0 {
                    self.tiles[idx].object = Object::Hint(mine_count as _);
                }
            }
        }
    }
}
