use crate::command::Error;
use crate::config::Config;
use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Entry {
    pub name: String,
    pub alias: String,
    pub timestamp: u64,
}

type Selection<'i> = BTreeSet<(usize, &'i Entry)>;

impl Entry {
    pub fn true_name(&self) -> String {
        self.name.replace(&std::env::var("HOME").unwrap_or("~".to_string()), "~")
    }

    pub fn info(&self, cfg: &Config) {
        let mut file = cfg.registry().to_path_buf();
        file.push(&self.alias);
        file.push("meta");
        let text = match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(_) => panic!(),
        };
        println!("{}", text);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Entries {
    contents: Vec<Entry>,
    blocks: Vec<usize>,
}

impl Entries {
    pub fn from_file(file: &std::path::Path) -> Result<Self, Error> {
        let contents = match std::fs::read_to_string(file) {
            Ok(contents) => contents,
            Err(_) => {
                return Err(Error::HistoryNotReadable(
                    file.to_str().unwrap().to_string(),
                ))
            }
        };
        let sep = regex::Regex::new(r"\n{2,}").unwrap();
        let mut idx = 1;
        let mut entries = Self::default();
        for block in sep.split(&contents).collect::<Vec<_>>().into_iter().rev() {
            if block == "" {
                continue;
            }
            entries.blocks.push(idx);
            for entry in block.split("\n").collect::<Vec<_>>().into_iter().rev() {
                if entry == "" {
                    continue;
                }
                let mut data = entry.split("|");
                let alias = data
                    .next()
                    .ok_or_else(|| Error::MissingData(entry.to_string(), idx, "alias"))?
                    .to_string();
                let name = data
                    .next()
                    .ok_or_else(|| Error::MissingData(entry.to_string(), idx, "name"))?
                    .to_string();
                let timestamp_str = data
                    .next()
                    .ok_or_else(|| Error::MissingData(entry.to_string(), idx, "timestamp"))?;
                let timestamp = timestamp_str
                    .parse::<u64>()
                    .map_err(|_| Error::CorruptedTimestamp(timestamp_str.to_string()))?;
                idx += 1;
                entries.contents.push(Entry { name, alias, timestamp });
            }
        }
        entries.blocks.push(idx);
        Ok(entries)
    }

    pub fn info<'i>(&self, cfg: &Config, selection: &Selection<'i>) {
        for (_, e) in selection {
            e.info(cfg);
        }
    }

    pub fn delete<'i>(&self, cfg: &Config, selection: &Selection<'i>) {
        unimplemented!()
    }

    pub fn restore<'i>(&self, cfg: &Config, selection: &Selection<'i>) {
        unimplemented!()
    }
}

pub trait Select {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>);
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

    pub fn as_block(self) -> Block {
        Block {
            start: self.start,
            end: self.end,
        }
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
pub struct Block {
    start: usize,
    end: usize,
}

impl Block {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
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
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>) {
        for (i, e) in entries.contents.iter().enumerate() {
            if !selection.contains(&(i, e)) && self.0.is_match(&e.name) {
                selection.insert((i, e));
            }
        }
    }
}
impl Select for Time {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>) {
        unimplemented!()
    }
}
impl Select for Index {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>) {
        let max = entries.contents.len() - 1;
        for i in self.start..=self.end.min(max) {
            selection.insert((i, &entries.contents[i]));
        }
    }
}
impl Select for Fzf {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>) {
        unimplemented!()
    }
}
impl Select for Block {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>) {
        unimplemented!()
    }
}

impl Select for Selector {
    fn select<'i>(&self, entries: &'i Entries, selection: &mut Selection<'i>) {
        for s in &self.0 {
            s.select(entries, selection);
        }
    }
}
