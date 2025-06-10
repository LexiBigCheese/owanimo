use alloc::borrow::Cow;
use hashbrown::HashSet;

use crate::{Board, BoardHandle, RefGroups, Scorer};

/// Display the score as AxB, multiply the numbers together to get the actual score.
///
/// Make sure you filter popped to the groups of pieces actually popped (when using standard Scorers).
///
/// Feel free to make your own version of this function using the provided parts.
///
/// Also, feel free to use `&()` or `&0` (or `&my_u64_score`) as scorers!
pub fn score<B: Board>(
    board: &B,
    popped: &RefGroups<B::Handle>,
    pieces_cleared: &impl Scorer<B>,
    point_bonus: &impl Scorer<B>,
    chain_power: &impl Scorer<B>,
    color_bonus: &impl Scorer<B>,
    group_bonus: &impl Scorer<B>,
) -> (u64, u64) {
    let score = 10 * pieces_cleared.score(board, popped) + point_bonus.score(board, popped);
    let multiplier = chain_power.score(board, popped)
        + color_bonus.score(board, popped)
        + group_bonus.score(board, popped);
    (score, multiplier)
}

pub struct StandardScorer<
    'a,
    B: Board,
    PC: Scorer<B>,
    PB: Scorer<B>,
    CP: Scorer<B>,
    CB: Scorer<B>,
    GB: Scorer<B>,
> {
    pub pieces_cleared: &'a PC,
    pub point_bonus: &'a PB,
    pub chain_power: &'a CP,
    pub color_bonus: &'a CB,
    pub group_bonus: &'a GB,
    pub phantom: core::marker::PhantomData<B>,
}

impl<'a, B: Board, PC: Scorer<B>, PB: Scorer<B>, CP: Scorer<B>, CB: Scorer<B>, GB: Scorer<B>>
    Scorer<B> for StandardScorer<'a, B, PC, PB, CP, CB, GB>
{
    fn score(&self, board: &B, popped: &RefGroups<<B as Board>::Handle>) -> u64 {
        let (a, b) = score(
            board,
            popped,
            self.pieces_cleared,
            self.point_bonus,
            self.chain_power,
            self.group_bonus,
            self.color_bonus,
        );
        a * b
    }
}

///Note that if you use Sun or Point pieces, you should implement your own Scorer here
pub struct TrivialPiecesCleared;

impl<B: Board> Scorer<B> for TrivialPiecesCleared {
    fn score(&self, _board: &B, popped: &RefGroups<<B as Board>::Handle>) -> u64 {
        popped.groups.iter().map(|x| x.len() as u64).sum()
    }
}

pub trait ColorBoard: Board {
    type Color: BoardHandle;
    fn color(&self, handle: &Self::Handle) -> Option<Self::Color>;
}

///Note: Only checks the first color of every group
pub struct ColorBonusTable<'a> {
    pub table: &'a [u64],
}

impl<'a, B: ColorBoard> Scorer<B> for ColorBonusTable<'a> {
    fn score(&self, board: &B, popped: &RefGroups<<B as Board>::Handle>) -> u64 {
        let mut colors: HashSet<B::Color> = Default::default();
        for g in &popped.groups {
            if let Some(color) = g.iter().next().and_then(|handle| board.color(handle)) {
                colors.get_or_insert(color);
            };
        }
        *self
            .table
            .get(colors.len())
            .or(self.table.last())
            .unwrap_or(&0)
    }
}

pub trait GroupBoard: Board {
    fn consider_for_group_bonus(&self, group: &HashSet<Self::Handle>) -> bool;
}

pub trait GroupFromColorBoard: ColorBoard {}

impl<T: GroupFromColorBoard> GroupBoard for T {
    fn consider_for_group_bonus(&self, group: &HashSet<Self::Handle>) -> bool {
        group.iter().next().and_then(|h| self.color(h)).is_some()
    }
}

pub struct GroupBonusTable<'a> {
    pub table: &'a [u64],
}

impl<'a, B: GroupBoard> Scorer<B> for GroupBonusTable<'a> {
    fn score(&self, board: &B, popped: &RefGroups<<B as Board>::Handle>) -> u64 {
        let mut group_bonus = 0u64;
        for g in &popped.groups {
            if board.consider_for_group_bonus(g) {
                group_bonus += self.table.get(g.len()).or(self.table.last()).unwrap_or(&0);
            }
        }
        group_bonus
    }
}

pub trait NuisanceBoard: Board {
    ///Will this being be sucked into the otherworld if a neighboring being was banished?
    ///
    /// Usually, the beings weak to this behaviour look transparent. and may also be called garbage or ojama.
    fn nuisance(&self, handle: &Self::Handle) -> bool;
}

impl<'a, H: BoardHandle> RefGroups<'a, H> {
    ///The Third part of the Owanimo spell, a Side Effect if you will, but an Important one.
    ///
    ///Use this function once you've opened portals to the otherworld with `owanimo_pop`
    pub fn owanimo_nuisance<B: NuisanceBoard + Board<Handle = H>>(
        &self,
        board: &B,
    ) -> RefGroups<H> {
        self.into_iter()
            .chain(
                board
                    .tiles()
                    .filter(|p| board.nuisance(p))
                    .filter(|p| board.neighbors(p).any(|n| self.test(&n)))
                    .map(|p| [p].into_iter().collect())
                    .map(Cow::Owned),
            )
            .collect()
    }
}
