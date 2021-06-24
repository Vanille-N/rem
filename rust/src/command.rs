use crate::select::{self, Entries, Entry, Select};
use std::collections::BTreeSet;
use std::fmt;


macro_rules! esc {
    ( $( $c:tt );+ ) => {{
        format!("\x1b[{}m", stringify!($( $c );+))
    }};
    ( ) => {
        esc![0]
    };
}

const RED: u8 = 91;
const BOLD: u8 = 1;
const ITAL: u8 = 3;
const GREEN: u8 = 92;
const BLUE: u8 = 94;
const YELLOW: u8 = 93;
const GREY: u8 = 97;
const PURPLE: u8 = 95;

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
        match regex::Regex::new(&self.0) {
            Ok(re) => Ok(select::Pattern::new(re)),
            Err(regex::Error::Syntax(s)) => Err(Error::InvalidRegexSyntax(self.0, s)),
            Err(_) => Err(Error::RegexFailure(self.0)),
        }
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
            "" => 1,
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
        if start > end {
            eprintln!("{}", Warning::EmptyRange(self.0.clone(), start as u64, end as u64));
        }
        Ok(select::Index::new(start, end))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Time(String);
impl Time {
    fn delta_time(s: &str) -> Result<u64, Error> {
        let mut acc = 0;
        let mut curr = None;
        for c in s.chars() {
            match c {
                '0'..='9' => {
                    let base = curr.unwrap_or(0) * 10;
                    let add = c.to_digit(10).unwrap() as u64;
                    curr = Some(base + add);
                }
                's' | 'm' | 'h' | 'D' | 'W' | 'M' | 'Y' => {
                    let multiplier = match c {
                        's' => 1,
                        'm' => 60,
                        'h' => 60 * 60,
                        'D' => 60 * 60 * 24,
                        'W' => 60 * 60 * 24 * 7,
                        'M' => 60 * 60 * 24 * 30,
                        'Y' => 60 * 60 * 24 * 365,
                        _ => unreachable!(),
                    };
                    acc += curr.unwrap_or(1) * multiplier;
                    curr = None;
                }
                c if c.is_whitespace() => {}
                _ => return Err(Error::WrongDuration(c)),
            }
        }
        Ok(acc)
    }

    pub fn into(self) -> Result<select::Time, Error> {
        let mut parts = self.0.split(':');
        let start = parts.next();
        let end = parts.next();
        if parts.next().is_some() {
            return Err(Error::ThreePartRange(self.0.clone()));
        }
        let start = match start.unwrap() {
            "" => 0,
            s => Self::delta_time(s)?,
        };
        let end = match end {
            None => start,
            Some("") => u64::MAX,
            Some(s) => Self::delta_time(s)?,
        };
        if start > end {
            eprintln!("{}", Warning::EmptyRange(self.0.clone(), start, end));
        }
        Ok(select::Time::new(start, end))
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
    active: bool,
    pat: Vec<Pattern>,
    idx: Vec<Index>,
    time: Vec<Time>,
    fzf: bool,
}

#[derive(Debug)]
pub enum Error {
    NonExclusiveCmd(&'static str, &'static str),
    TooManyArgs(&'static str, Vec<String>),
    EmptySelectorList(&'static str),
    ThreePartRange(String),
    InvalidIndex(String),
    UnknownArg(String),
    UndoUselessSelector(Selector),
    RemoveUselessSelector(Selector),
    WrongDuration(char),
    InvalidRegexSyntax(String, String),
    RegexFailure(String),
}

#[derive(Debug)]
enum Warning {
    EmptyRange(String, u64, u64),
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Warning::EmptyRange(original, start, end) => {
                writeln!(f, "{}Empty range cannot match{}", esc![BOLD;RED], esc![])?;
                writeln!(f, "\tRange pattern '{}' interpreted as {}..={} is useless since it will never match", original, start, end)?;
            }
        }
        Ok(())
    }
}

macro_rules! do_take_while {
    ( $args:expr, $label:expr, $( $insertion:tt )+ ) => {{
        let mut first = true;
        while let Some(s) = $args.peek() {
            if first || s.as_ref().chars().next() != Some('-') {
                $( $insertion )+(String::from($args.next().unwrap().as_ref()));
            } else {
                break;
            }
            first = false;
        }
        if first {
            return Err(Error::EmptySelectorList($label));
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
                    "--pat" | "-P" => do_take_while!(args, "pat", selector.add_pat),
                    "--idx" | "-I" => do_take_while!(args, "idx", selector.add_idx),
                    "--time" | "-T" => do_take_while!(args, "time", selector.add_time),
                    "--sandbox" | "-S" => sandbox = true,
                    "--overwrite" | "-O" => overwrite = true,
                    "--" => break,
                    _ => {
                        if arg.as_ref().starts_with('-') {
                            return Err(Error::UnknownArg(arg.as_ref().to_string()));
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
                    return Err(Error::TooManyArgs("undo", pos_args));
                }
                if selector.active {
                    return Err(Error::UndoUselessSelector(selector));
                }
                Action::Undo
            }
            (_, _, Some(ed)) => {
                if !pos_args.is_empty() {
                    return Err(Error::TooManyArgs(ed.as_str(), pos_args));
                }
                Action::Edit(Some(ed), selector)
            }
            _ => {
                if pos_args.is_empty() {
                    Action::Edit(None, selector)
                } else {
                    if selector.active {
                        return Err(Error::RemoveUselessSelector(selector));
                    }
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
        self.active = true;
    }

    pub fn add_pat(&mut self, pat: String) {
        self.pat.push(Pattern(pat));
        self.active = true;
    }

    pub fn add_idx(&mut self, idx: String) {
        self.idx.push(Index(idx));
        self.active = true;
    }

    pub fn add_time(&mut self, time: String) {
        self.time.push(Time(time));
        self.active = true;
    }

    pub fn into(self) -> Result<select::Selector, Error> {
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

macro_rules! assert_matches {
    ( $obj:expr, $target:pat ) => {{
        let obj = $obj;
        if !matches!(obj, $target) {
            panic!(
                "{:?} should match {:?} but does not",
                obj,
                stringify!($target)
            );
        }
    }};
}

mod test {
    use super::*;
    use crate::select;
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
                    active: true,
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

    #[test]
    fn selectors_capture() {
        let ended = Command::parse(&["-I", "1", "2", "3", "-P", ""]).unwrap();
        assert_eq!(
            ended.action,
            Action::Edit(
                None,
                Selector {
                    active: true,
                    fzf: false,
                    idx: vec![
                        Index("1".to_string()),
                        Index("2".to_string()),
                        Index("3".to_string())
                    ],
                    pat: vec![Pattern("".to_string())],
                    time: vec![],
                }
            )
        );
    }

    #[test]
    fn errors() {
        let non_exclusive1 = Command::parse(&["--help", "--rest", "-P", "foo"]);
        assert_matches!(non_exclusive1, Err(Error::NonExclusiveCmd(_, _)));
        let non_exclusive2 = Command::parse(&["--help", "--undo"]);
        assert_matches!(non_exclusive2, Err(Error::NonExclusiveCmd(_, _)));
        let non_exclusive3 = Command::parse(&["--rest", "--del"]);
        assert_matches!(non_exclusive3, Err(Error::NonExclusiveCmd(_, _)));
        let too_many = Command::parse(&["foo", "bar", "--undo"]);
        assert_matches!(too_many, Err(Error::TooManyArgs(_, _)));
        let empty_sel = Command::parse(&["-P"]);
        assert_matches!(empty_sel, Err(Error::EmptySelectorList(_)));
        //let three_part = Command::parse(&["-I", "1:3:"]);
        //assert_matches!(three_part, Err(Error::ThreePartRange(_)));
        //let invalid = Command::parse(&["--idx", "a:4"]);
        //assert_matches!(invalid, Err(Error::InvalidIndex(_)));
        let unknown = Command::parse(&["--foo"]);
        assert_matches!(unknown, Err(Error::UnknownArg(_)));
        let useless1 = Command::parse(&["-F", "--undo"]);
        assert_matches!(useless1, Err(Error::UndoUselessSelector(_)));
        let useless2 = Command::parse(&["foo.txt", "-I", "3"]);
        assert_matches!(useless2, Err(Error::RemoveUselessSelector(_)));
    }

    #[test]
    fn selector_idx() {
        assert_eq!(
            Index("1".to_string()).into().unwrap(),
            select::Index::new(1, 1)
        );
        assert_eq!(
            Index("1:4".to_string()).into().unwrap(),
            select::Index::new(1, 4)
        );
        assert_eq!(
            Index(":15".to_string()).into().unwrap(),
            select::Index::new(1, 15)
        );
        assert_matches!(
            Index("1:2:3".to_string()).into(),
            Err(Error::ThreePartRange(_))
        );
        assert_matches!(Index("a:4".to_string()).into(), Err(Error::InvalidIndex(_)));
    }

    #[test]
    fn selector_time() {
        assert_eq!(
            Time("3D4m:1Y3s30W".to_string()).into().unwrap(),
            select::Time::new(
                4 * 60 + 3 * 24 * 60 * 60,
                3 + 30 * 7 * 24 * 60 * 60 + 1 * 365 * 24 * 60 * 60
            )
        );
        assert_eq!(
            Time("h:".to_string()).into().unwrap(),
            select::Time::new(1 * 60 * 60, u64::MAX)
        );
        assert_eq!(
            Time("13M".to_string()).into().unwrap(),
            select::Time::new(
                13 * 30 * 24 * 60 * 60,
                13 * 30 * 24 * 60 * 60
            )
        );
    }
}
