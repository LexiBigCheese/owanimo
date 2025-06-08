use owanimo::{
    Board,
    standard::{ColorBoard, NuisanceBoard},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Tile {
    #[default]
    Air,
    Nuisance,
    Green,
    Red,
    Blue,
    Yellow,
    Purple,
}

struct TileBoard {
    items: [[Tile; 6]; 12],
}

impl TileBoard {
    fn get(&self, (x, y): (usize, usize)) -> Tile {
        *self
            .items
            .get(y)
            .and_then(|row| row.get(x))
            .unwrap_or(&Tile::Air)
    }
    fn set(&mut self, (x, y): (usize, usize), to: Tile) {
        let mut noop = Tile::Air;
        *self
            .items
            .get_mut(y)
            .and_then(|row| row.get_mut(x))
            .unwrap_or(&mut noop) = to;
    }
}

impl Board for TileBoard {
    type Handle = (usize, usize);

    fn tiles(&self) -> impl Iterator<Item = Self::Handle> {
        (0..6).flat_map(move |x| (0..12).map(move |y| (x, y)))
    }

    fn neighbors(&self, handle: &Self::Handle) -> impl Iterator<Item = Self::Handle> {
        let (x, y) = *handle;
        [
            if x > 0 { Some((x - 1, y)) } else { None },
            if y > 0 { Some((x, y - 1)) } else { None },
            if x < 5 { Some((x + 1, y)) } else { None },
            if y < 11 { Some((x, y + 1)) } else { None },
        ]
        .into_iter()
        .flatten()
    }

    fn connects(&self, a: &Self::Handle, b: &Self::Handle) -> bool {
        use Tile::*;
        match (self.get(*a), self.get(*b)) {
            (Air | Nuisance, Air | Nuisance) => false,
            (x, y) => x == y,
        }
    }
}

impl NuisanceBoard for TileBoard {
    fn nuisance(&self, handle: &Self::Handle) -> bool {
        self.get(*handle) == Tile::Nuisance
    }
}

impl ColorBoard for TileBoard {
    type Color = Tile;
    fn color(&self, handle: &Self::Handle) -> Option<Self::Color> {
        use Tile::*;
        match self.get(*handle) {
            Air | Nuisance => None,
            x => Some(x),
        }
    }
}
