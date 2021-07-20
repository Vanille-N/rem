use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Entry {
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
    fn select<'i>(&self, entries: &'i Entries, selection: &mut BTreeSet<(usize, &'i Entry)>);
}

#[derive(Debug)]
pub struct Index {
    start: usize,
    end: usize,
}

impl Index {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct Time {
    start: u64,
    end: u64,
}

impl Time {
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct Pattern(regex::Regex);

impl Pattern {
    pub fn new(re: regex::Regex) -> Self {
        Self(re)
    }
}

#[derive(Debug)]
pub struct Fzf {}

#[derive(Debug)]
pub struct Group(usize);

impl Group {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Default)]
pub struct Selector(Vec<Box<dyn Select + 'static>>);

impl Selector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<S>(&mut self, sel: S)
    where
        S: Select + 'static,
    {
        self.0.push(Box::new(sel));
    }
}
impl Select for Pattern {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut BTreeSet<(usize, &'i Entry)>) {
        for (i, e) in entries.contents.iter().enumerate() {
            if !selection.contains(&(i, e)) && self.0.is_match(&e.name) {
                selection.insert((i, e));
            }
        }
    }
}
impl Select for Time {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut BTreeSet<(usize, &'i Entry)>) {
        unimplemented!()
    }
}
impl Select for Index {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut BTreeSet<(usize, &'i Entry)>) {
        let max = entries.contents.len() - 1;
        for i in self.start..=self.end.min(max) {
            selection.insert((i, &entries.contents[i]));
        }
    }
}
impl Select for Fzf {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut BTreeSet<(usize, &'i Entry)>) {
        unimplemented!()
    }
}
impl Select for Group{
    fn select<'i>(&self, entries: &'i Entries, selection: &mut BTreeSet<(usize, &'i Entry)>) {
        unimplemented!()
    }
}
