use crate::select::{self, Entries, Entry, Select};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Command {
    action: Action,
    sandbox: bool,
    overwrite: bool,
    critical: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Remove(Vec<File>),
    Edit(Option<Editor>, Selector),
    Undo,
    Help(Vec<Help>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Help(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern(String);
impl Pattern {
    pub fn into(self) -> Result<select::Pattern, Error> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index(String);
impl Index {
    pub fn into(self) -> Result<select::Index, Error> {
        let mut parts = self.0.split(':');
        let start = parts.next();
        let end = parts.next();
        if parts.next().is_some() {
            return Err(Error::ThreePartRange(self.0.clone()));
        }
        let start = match start.unwrap() {
            "" => 0,
            s => match s.parse::<usize>() {
                Ok(n) => n,
                Err(_) => return Err(Error::InvalidIndex(s.to_string())),
            },
        };
        let end = match end {
            None => start,
            Some("") => std::usize::MAX,
            Some(s) => match s.parse::<usize>() {
                Ok(n) => n,
                Err(_) => return Err(Error::InvalidIndex(s.to_string())),
            },
        };
        Ok(select::Index::new(start, end))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Time(String);
impl Time {
    pub fn into(self) -> Result<select::Time, Error> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Editor {
    Delete,
    Restore,
    Info,
}

impl Editor {
    pub fn as_str(self) -> &'static str {
        match self {
            Editor::Delete => "del",
            Editor::Restore => "rest",
            Editor::Info => "info",
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Selector {
    pat: Vec<Pattern>,
    idx: Vec<Index>,
    time: Vec<Time>,
    fzf: bool,
}

#[derive(Debug)]
pub enum Error {
    NonExclusiveCmd(&'static str, &'static str),
    TooManyArgs(&'static str),
    ThreePartRange(String),
    InvalidIndex(String),
}

macro_rules! do_take_while {
    ( $args:expr, $( $insertion:tt )+ ) => {{
        let mut first = true;
        while let Some(s) = $args.peek() {
            if first || s.as_ref().chars().next() != Some('-') {
                $( $insertion )+(String::from($args.next().unwrap().as_ref()));
            } else {
                break;
            }
            first = false;
        }
    }}
}

impl Command {
    pub fn argparse() -> Result<Self, Error> {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse<I, S>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut pos_args = Vec::new();
        let mut selector = Selector::default();
        let mut help = false;
        let mut undo = false;
        let mut editor = OnceEd::new();
        let mut sandbox = false;
        let mut overwrite = false;
        let mut args = args.into_iter().peekable();
        loop {
            match args.next() {
                None => break,
                Some(arg) => match arg.as_ref() {
                    "--info" | "-i" => editor.set(Editor::Info)?,
                    "--help" | "-h" => help = true,
                    "--undo" | "-u" => undo = true,
                    "--rest" | "-r" => editor.set(Editor::Restore)?,
                    "--del" | "-d" => editor.set(Editor::Delete)?,
                    "--fzf" | "-F" => selector.add_fzf(),
                    "--pat" | "-P" => do_take_while!(args, selector.add_pat),
                    "--idx" | "-I" => do_take_while!(args, selector.add_idx),
                    "--time" | "-T" => do_take_while!(args, selector.add_time),
                    "--sandbox" | "-S" => sandbox = true,
                    "--overwrite" | "-O" => overwrite = true,
                    "--" => break,
                    _ => {
                        if arg.as_ref().starts_with('-') {
                            panic!("Unknown argument '{}'", arg.as_ref());
                        }
                        pos_args.push(arg.as_ref().to_string());
                    }
                },
            }
        }
        for arg in args {
            // drain remaining args as positional (encountered '--')
            pos_args.push(arg.as_ref().to_string());
        }
        let action = match (help, undo, editor.into_inner()) {
            // Incompatibilities
            (true, true, _) => return Err(Error::NonExclusiveCmd("help", "undo")),
            (true, _, Some(ed)) => return Err(Error::NonExclusiveCmd("help", ed.as_str())),
            (_, true, Some(ed)) => return Err(Error::NonExclusiveCmd("undo", ed.as_str())),
            // Ok
            (true, _, _) => Action::Help(pos_args.into_iter().map(Help).collect()),
            (_, true, _) => {
                if !pos_args.is_empty() {
                    return Err(Error::TooManyArgs("undo"));
                }
                Action::Undo
            }
            (_, _, Some(ed)) => {
                if !pos_args.is_empty() {
                    return Err(Error::TooManyArgs(ed.as_str()));
                }
                Action::Edit(Some(ed), selector)
            }
            _ => {
                if pos_args.is_empty() {
                    Action::Edit(None, selector)
                } else {
                    Action::Remove(pos_args.into_iter().map(File).collect())
                }
            }
        };
        let critical = !matches!(
            &action,
            Action::Edit(None, _) | Action::Edit(Some(Editor::Info), _) | Action::Help(_)
        );
        Ok(Self {
            action,
            sandbox,
            overwrite,
            critical,
        })
    }
}

impl Selector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fzf(&mut self) {
        self.fzf = true;
    }

    pub fn add_pat(&mut self, pat: String) {
        self.pat.push(Pattern(pat));
    }

    pub fn add_idx(&mut self, idx: String) {
        self.idx.push(Index(idx));
    }

    pub fn add_time(&mut self, time: String) {
        self.time.push(Time(time));
    }

    pub fn into(self) -> Result<select::Selector<'static>, Error> {
        let mut sel = select::Selector::new();
        for p in self.pat {
            sel.push(p.into()?);
        }
        for t in self.time {
            sel.push(t.into()?);
        }
        for i in self.idx {
            sel.push(i.into()?);
        }
        if self.fzf {
            sel.push(select::Fzf {});
        }
        Ok(sel)
    }
}

#[derive(Debug)]
struct OnceEd {
    data: Option<Editor>,
}

impl OnceEd {
    fn new() -> Self {
        Self { data: None }
    }

    fn set(&mut self, new: Editor) -> Result<(), Error> {
        if self.data.is_none() {
            self.data = Some(new);
            Ok(())
        } else {
            Err(Error::NonExclusiveCmd(
                self.data.unwrap().as_str(),
                new.as_str(),
            ))
        }
    }

    fn into_inner(self) -> Option<Editor> {
        self.data
    }
}

mod test {
    use super::*;
    #[test]
    fn flags_are_detected() {
        let both = Command::parse(&["-S", "-O", "-F", "-I", "3-"]).unwrap();
        assert!(both.sandbox);
        assert!(both.overwrite);
        let neither = Command::parse(&["-F", "-I", "3-"]).unwrap();
        assert!(!neither.sandbox);
        assert!(!neither.overwrite);
    }

    #[test]
    fn command_is_detected() {
        let help = Command::parse(&["--help", "cmd", "-F", "main"]).unwrap();
        assert_eq!(
            help.action,
            Action::Help(vec![Help("cmd".to_string()), Help("main".to_string())])
        );
        let del = Command::parse(&["-d", "-I", "3:7"]).unwrap();
        assert_eq!(
            del.action,
            Action::Edit(
                Some(Editor::Delete),
                Selector {
                    fzf: false,
                    idx: vec![Index("3:7".to_string())],
                    pat: vec![],
                    time: vec![]
                }
            )
        );
        let undo = Command::parse(&["--undo"]).unwrap();
        assert_eq!(undo.action, Action::Undo);
        let remove = Command::parse(&["foo.txt", "bar.sh"]).unwrap();
        assert_eq!(
            remove.action,
            Action::Remove(vec![
                File("foo.txt".to_string()),
                File("bar.sh".to_string())
            ])
        );
    }
}
