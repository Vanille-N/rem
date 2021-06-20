use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    index: usize,
    name: String,
    alias: String,
    timestamp: u64,
}

pub trait Select {
    fn select(&self, entries: &[Entry], selection: &mut BTreeSet<Entry>);
}

#[derive(Debug)]
pub enum IndexSelector {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub enum TimeSelector {
    start: u64,
    end: u64,
}


#[derive(Debug)]
pub struct PatternSelector(String);

#[derive(Debug)]
pub struct FzfSelector {}
