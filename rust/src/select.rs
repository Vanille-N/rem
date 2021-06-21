use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Entry {
    index: usize,
    name: String,
    alias: String,
    timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Entries {
    contents: Vec<Entry>,
    blocks: Vec<usize>,
}

pub trait Select {
    fn select(&self, entries: &Entries, selection: &mut BTreeSet<&Entry>);
}

#[derive(Debug)]
pub struct Index {
    start: usize,
    end: usize,
}

impl Index {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
        }
    }
}

#[derive(Debug)]
pub struct Time {
    start: u64,
    end: u64,
}

#[derive(Debug)]
pub struct Pattern(String);

#[derive(Debug)]
pub struct Fzf {}

#[derive(Default)]
pub struct Selector<'s>(Vec<Box<dyn Select + 's>>);

impl<'s> Selector<'s> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<S>(&mut self, sel: S)
        where S: Select + 's {
            self.0.push(Box::new(sel));
    }
}
impl Select for Pattern {
    fn select(&self, entries: &Entries, selection: &mut BTreeSet<&Entry>) {
        unimplemented!()
    }
}
impl Select for Time {
    fn select(&self, entries: &Entries, selection: &mut BTreeSet<&Entry>) {
        unimplemented!()
    }
}
impl Select for Index {
    fn select(&self, entries: &Entries, selection: &mut BTreeSet<&Entry>) {
        unimplemented!()
    }
}
impl Select for Fzf {
    fn select(&self, entries: &Entries, selection: &mut BTreeSet<&Entry>) {
        unimplemented!()
    }
}
