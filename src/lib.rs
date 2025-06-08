#![no_std]

#[cfg(feature = "standard")]
pub mod standard;

extern crate alloc;

use alloc::vec::Vec;
use hashbrown::HashSet;

pub trait Board {
    type Handle: Copy + Clone + core::hash::Hash + Eq + Default;
    fn tiles(&self) -> impl Iterator<Item = Self::Handle>;
    fn neighbors(&self, handle: &Self::Handle) -> impl Iterator<Item = Self::Handle>;
    fn connects(&self, a: &Self::Handle, b: &Self::Handle) -> bool;
    ///The first part of the Owanimo spell, finds groups of beings on a board
    ///To get the second part of the spell, do `groups.as_ref().owanimo_pop()`
    fn owanimo_grouper(&self) -> Groups<Self::Handle> {
        let mut groups = Groups::default();
        for tile in self.tiles() {
            let mut me_group: HashSet<Self::Handle> = [tile].into_iter().collect();
            for neighbor in self.neighbors(&tile) {
                if self.connects(&tile, &neighbor) {
                    if let Some(x) = groups.find(&neighbor) {
                        me_group.extend(x);
                    }
                }
            }
            groups.push(me_group);
        }
        groups
    }
}

#[derive(Default, Clone)]
pub struct Groups<H: Copy + Clone + core::hash::Hash + Eq + Default> {
    pub groups: Vec<HashSet<H>>,
}

#[derive(Default, Clone)]
pub struct RefGroups<'a, H: Copy + Clone + core::hash::Hash + Eq + Default> {
    pub groups: Vec<alloc::borrow::Cow<'a, HashSet<H>>>,
}

impl<H: Copy + Clone + core::hash::Hash + Eq + Default> Groups<H> {
    pub fn find(&mut self, handle: &H) -> Option<HashSet<H>> {
        self.groups.extract_if(.., |g| g.contains(handle)).next()
    }
    pub fn push(&mut self, group: HashSet<H>) {
        self.groups.push(group);
    }
    pub fn as_ref(&self) -> RefGroups<H> {
        self.groups.iter().collect()
    }
}

impl<'a, H: Copy + Clone + core::hash::Hash + Eq + Default>
    FromIterator<alloc::borrow::Cow<'a, HashSet<H>>> for RefGroups<'a, H>
{
    fn from_iter<T: IntoIterator<Item = alloc::borrow::Cow<'a, HashSet<H>>>>(iter: T) -> Self {
        RefGroups {
            groups: iter.into_iter().collect(),
        }
    }
}

impl<'a, H: Copy + Clone + core::hash::Hash + Eq + Default> FromIterator<&'a HashSet<H>>
    for RefGroups<'a, H>
{
    fn from_iter<T: IntoIterator<Item = &'a HashSet<H>>>(iter: T) -> Self {
        RefGroups {
            groups: iter.into_iter().map(alloc::borrow::Cow::Borrowed).collect(),
        }
    }
}

impl<'a, 'b: 'a, H: Copy + Clone + core::hash::Hash + Eq + Default> IntoIterator
    for &'b RefGroups<'a, H>
{
    type Item = alloc::borrow::Cow<'a, HashSet<H>>;
    type IntoIter =
        core::iter::Cloned<<&'b [alloc::borrow::Cow<'a, HashSet<H>>] as IntoIterator>::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.groups).into_iter().cloned()
    }
}

impl<'a, H: Copy + Clone + core::hash::Hash + Eq + Default> RefGroups<'a, H> {
    pub fn find(&self, handle: &H) -> Option<alloc::borrow::Cow<HashSet<H>>> {
        self.groups
            .iter()
            .filter(|g| g.contains(handle))
            .map(Clone::clone)
            .next()
    }
    pub fn test(&self, handle: &H) -> bool {
        self.groups
            .iter()
            .filter(|g| g.contains(handle))
            .next()
            .is_some()
    }
    ///Second part of the Owanimo spell, selects large enough groups of beings to banish to the otherworld
    ///
    /// Most mages call this step "popping", because the beings most used for this ritual POP when they are banished.
    ///
    /// For the optional third part of the spell, see `owanimo_nuisance`
    pub fn owanimo_pop(&self, pieces_to_pop: usize) -> RefGroups<H> {
        self.into_iter()
            .filter(|g| g.len() >= pieces_to_pop)
            .collect()
    }
}

pub trait Scorer<B: Board> {
    fn score(&self, board: &B, popped: &RefGroups<<B as Board>::Handle>) -> u64;
}

impl<B: Board> Scorer<B> for () {
    fn score(&self, _board: &B, _popped: &RefGroups<<B as Board>::Handle>) -> u64 {
        0
    }
}

impl<B: Board> Scorer<B> for u64 {
    fn score(&self, _board: &B, _popped: &RefGroups<<B as Board>::Handle>) -> u64 {
        *self
    }
}
