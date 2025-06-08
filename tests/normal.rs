use std::str::FromStr;

use owanimo::{
    Board,
    standard::{ColorBoard, NuisanceBoard},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
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

#[derive(Default, Clone, Copy)]
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

#[derive(Debug, Clone)]
enum ParseError {
    UnknownSymbol { at: (usize, usize), symbol: char },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownSymbol { at, symbol } => {
                write!(f, "Unknown Symbol {} at {},{}", symbol, at.0, at.1)?
            }
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

impl FromStr for TileBoard {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseError::*;
        let mut this = Self::default();
        let lines = s
            .lines()
            .filter(|x| !x.trim().is_empty())
            .take(12)
            .collect::<Vec<_>>();
        let start_line = lines.len() - 1;
        for (y_bottom, line) in lines.into_iter().enumerate() {
            let y = start_line - y_bottom;
            for (x, chr) in line.trim().chars().take(6).enumerate() {
                use Tile::*;
                let tile = match chr {
                    ' ' | '_' => Air,
                    'O' | 'o' | '0' => Nuisance,
                    'R' | 'r' => Red,
                    'G' | 'g' => Green,
                    'B' | 'b' => Blue,
                    'Y' | 'y' => Yellow,
                    'P' | 'p' => Purple,
                    c => {
                        return Err(UnknownSymbol {
                            at: (x, y),
                            symbol: c,
                        });
                    }
                };
                this.set((x, y), tile);
            }
        }
        Ok(this)
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

#[test]
fn integration_itself_works() -> Result<(), Box<dyn std::error::Error>> {
    let board = "
        ___
        ypo
        rgb
    "
    .parse::<TileBoard>()?;
    use Tile::{
        Air as A, Blue as B, Green as G, Nuisance as O, Purple as P, Red as R, Yellow as Y,
    };
    assert_eq!(
        &board.items[0..2],
        &[[R, G, B, A, A, A], [Y, P, O, A, A, A]]
    );
    assert_eq!(&board.items[2..12], &[[A; 6]; 10]);
    assert_eq!(board.color(&(0, 0)), Some(R));
    assert_eq!(board.color(&(3, 0)), None);
    Ok(())
}
