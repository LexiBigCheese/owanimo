use crate::{
    BanishBoard, Board, Scorer,
    gravity::GravityBoard,
    standard::{NuisanceBoard, StandardScorer},
};

pub trait QuickSimBoard: Board + BanishBoard + GravityBoard + NuisanceBoard + Sized {
    ///Output: ()
    fn quick_sim(
        &mut self,
        pieces_to_pop: usize,
        pc: &impl Scorer<Self>,
        pb: &impl Scorer<Self>,
        chain_power_table: &[u64],
        cb: &impl Scorer<Self>,
        gb: &impl Scorer<Self>,
    ) -> SimResult {
        let mut score = 0;
        let mut chain = 0;
        let mut pieces_cleared = 0;
        let mut max_pieces_at_once = 0;
        loop {
            self.fall();
            let scorer = StandardScorer {
                chain_power: chain_power_table
                    .get(chain as usize)
                    .or(chain_power_table.last())
                    .unwrap_or(&0),
                color_bonus: cb,
                group_bonus: gb,
                point_bonus: pb,
                pieces_cleared: pc,
                phantom: Default::default(),
            };
            let grps = self.owanimo_grouper();
            let binding = grps.as_ref();
            let binding = binding.owanimo_pop(pieces_to_pop);
            let pg = binding.owanimo_nuisance(self);
            let this_score = scorer.score(self, &pg);
            let this_pieces_cleared = pc.score(self, &pg);
            for g in &pg {
                for &p in g.iter() {
                    self.banish(p);
                }
            }
            score += this_score;
            pieces_cleared += this_pieces_cleared;
            max_pieces_at_once = max_pieces_at_once.max(this_pieces_cleared);
            if this_pieces_cleared == 0 {
                break;
            }
            chain += 1;
        }
        SimResult {
            score,
            chain,
            pieces_cleared,
            max_pieces_at_once,
        }
    }
}

pub struct SimResult {
    pub score: u64,
    pub chain: u64,
    pub pieces_cleared: u64,
    pub max_pieces_at_once: u64,
}
