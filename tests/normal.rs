use std::str::FromStr;

use owanimo::{
    Board, RefGroups, Scorer,
    gravity::{AutoGravityBoard, GravityBoard},
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct TileBoard {
    items: [[Tile; 6]; 12],
}

impl std::fmt::Debug for TileBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌──────┐")?;
        for line in self.items.iter().rev() {
            write!(f, "│")?;
            for chr in line {
                let to_write = match chr {
                    Tile::Air => " ",
                    Tile::Nuisance => "\x1B[0m●\x1B[0m",
                    Tile::Green => "\x1B[92m●\x1B[0m",
                    Tile::Red => "\x1B[91m●\x1B[0m",
                    Tile::Blue => "\x1B[94m●\x1B[0m",
                    Tile::Yellow => "\x1B[93m●\x1B[0m",
                    Tile::Purple => "\x1B[35m●\x1B[0m",
                };
                write!(f, "{}", to_write)?;
            }
            writeln!(f, "│")?;
        }
        writeln!(f, "└──────┘")?;
        Ok(())
    }
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
    fn pop(&mut self, refs: &RefGroups<(usize, usize)>) {
        for g in refs.into_iter() {
            for g in g.iter().copied() {
                self.set(g, Tile::Air);
            }
        }
    }
    fn getcol(&self, x: usize) -> [Tile; 12] {
        let mut vals = [Tile::Air; 12];
        for y in 0..12 {
            vals[y] = self.get((x, y));
        }
        vals
    }
    fn setcol(&mut self, x: usize, vs: [Tile; 12]) {
        for (y, v) in vs.into_iter().enumerate() {
            self.set((x, y), v);
        }
    }
    fn fall(&mut self) {
        for x in 0..6 {
            let mut col = self.getcol(x);
            let mut index = 0;
            'outer: loop {
                let cursor_a = index;
                //cursor starts at bottom, assumes itself to be air,
                while col[index] == Tile::Air {
                    index += 1;
                    if index == col.len() {
                        break 'outer;
                    };
                }
                let cursor_b = index;
                //now find how many are not
                while col[index] != Tile::Air {
                    index += 1;
                    if index == col.len() {
                        break;
                    }
                }
                //then rotate cursor_a..index leftwards to push the air bubbles to the top and repeat
                col[cursor_a..index].rotate_left(cursor_b - cursor_a);
                //and rotate the index too
                index -= cursor_b - cursor_a;
                if index == col.len() {
                    break;
                }
            }
            self.setcol(x, col);
        }
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

impl AutoGravityBoard for TileBoard {
    fn is_air(&self, handle: Self::Handle) -> bool {
        self.get(handle) == Tile::Air
    }
    fn mutate_columns(&mut self, mutater: impl Fn(&Self, &mut [Self::Handle])) {
        for x in 0..6 {
            let mut col = [(x, 0); 12];
            for y in 0..12 {
                col[y].1 = y;
            }
            mutater(&self, &mut col);
            let tiles_original = self.getcol(x);
            let mut tiles_new = [Tile::Air; 12];
            for ((_, y), set_me) in col.into_iter().zip(&mut tiles_new) {
                *set_me = tiles_original[y];
            }
            self.setcol(x, tiles_new);
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

#[test]
fn do_a_pop() -> Result<(), Box<dyn std::error::Error>> {
    let mut board = "rrrrb".parse::<TileBoard>()?;
    let original_groups = board.owanimo_grouper();
    let binding = original_groups.as_ref();
    let binding = binding.owanimo_pop(4);
    let popped_groups = binding.owanimo_nuisance(&board);
    board.pop(&popped_groups);
    assert_eq!("____b".parse::<TileBoard>()?, board);
    Ok(())
}

#[test]
fn do_a_chain() -> Result<(), Box<dyn std::error::Error>> {
    let mut board = "
    orbg
    rbgy
    rbgyo
    rbgyy
    "
    .parse::<TileBoard>()?;
    loop {
        board.fall();
        println!("{:?}", board);
        let grps = board.owanimo_grouper();
        let binding = grps.as_ref();
        let binding = binding.owanimo_pop(4);
        let pg = binding.owanimo_nuisance(&board);
        let pcs = owanimo::standard::TrivialPiecesCleared.score(&board, &pg);
        board.pop(&pg);
        println!("{:?}", board);
        if pcs == 0 {
            break;
        }
    }
    assert_eq!(board.items, [[Tile::Air; 6]; 12]);
    Ok(())
}

#[test]
fn do_a_chain_autogravity() -> Result<(), Box<dyn std::error::Error>> {
    let mut board = "
    orbg
    rbgy
    rbgyo
    rbgyy
    "
    .parse::<TileBoard>()?;
    loop {
        <TileBoard as GravityBoard>::fall(&mut board);
        println!("{:?}", board);
        let grps = board.owanimo_grouper();
        let binding = grps.as_ref();
        let binding = binding.owanimo_pop(4);
        let pg = binding.owanimo_nuisance(&board);
        let pcs = owanimo::standard::TrivialPiecesCleared.score(&board, &pg);
        board.pop(&pg);
        println!("{:?}", board);
        if pcs == 0 {
            break;
        }
    }
    assert_eq!(board.items, [[Tile::Air; 6]; 12]);
    Ok(())
}
