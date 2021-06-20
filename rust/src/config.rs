use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    root: PathBuf,
    history: PathBuf,
    lock: PathBuf,
    help: PathBuf,
}

#[derive(Debug)]
pub struct Command {
    action: Action,
    sandbox: bool,
    overwrite: bool,
    critical: bool,
}

#[derive(Debug)]
pub enum Action {
    Remove(FileList),
    Edit(Option<Editor>, Selector),
    Undo,
    Help(HelpList),
}

pub type FileList = Vec<String>;
pub type HelpList = Vec<String>;
pub type PatternList = Vec<String>;
pub type IndexList = Vec<String>;
pub type TimeList = Vec<String>;

#[derive(Debug)]
pub enum Editor {
    Delete,
    Restore,
    Info,
}

#[derive(Debug, Default)]
pub struct Selector {
    pat: PatternList,
    idx: IndexList,
    time: TimeList,
    fzf: bool,
}

macro_rules! do_take_while {
    ( $args:expr, $( $insertion:tt )+ ) => {{
        let mut first = true;
        while let Some(s) = $args.peek() {
            if first || s.chars().next() != Some('-') {
                $( $insertion )+($args.next().unwrap());
            } else {
                break;
            }
            first = false;
        }
    }}
}

impl Command {
    pub fn argparse(args: std::env::Args) -> Self {
        let mut pos_args = Vec::new();
        let mut selector = Selector::default();
        let mut help = false;
        let mut undo = false;
        let mut editor = Once::new();
        let mut sandbox = false;
        let mut overwrite = false;
        let mut args = args.into_iter().skip(1).peekable();
        loop {
            match args.next() {
                None => break,
                Some(arg) => match arg.as_str() {
                    "--info" | "-i" => editor.set(Editor::Info),
                    "--help" | "-h" => help = true,
                    "--undo" | "-u" => undo = true,
                    "--rest" | "-r" => editor.set(Editor::Restore),
                    "--del" | "-d" => editor.set(Editor::Delete),
                    "--fzf" | "-F" => selector.add_fzf(),
                    "--pat" | "-P" => do_take_while!(args, selector.add_pat),
                    "--idx" | "-I" => do_take_while!(args, selector.add_idx),
                    "--time" | "-T" => do_take_while!(args, selector.add_time),
                    "--sandbox" | "-S" => sandbox = true,
                    "--overwrite" | "-O" => overwrite = true,
                    "--" => {
                        while let Some(arg) = args.next() {
                            pos_args.push(arg);
                        }
                    }
                    _ => {
                        if arg.chars().next() == Some('-') {
                            panic!("Unknown argument '{}'", arg);
                        }
                        pos_args.push(arg);
                    }
                },
            }
        }
        let action = match (help, undo, editor.into_inner()) {
            // Incompatibilities
            (true, true, _) => panic!("Help and Undo must be exclusive"),
            (true, _, Some(ed)) => panic!("Help and {:?} must be exclusive", ed),
            (_, true, Some(ed)) => panic!("Undo and {:?} must be exclusive", ed),
            // Ok
            (true, _, _) => Action::Help(pos_args),
            (_, true, _) => {
                if !pos_args.is_empty() {
                    panic!("Undo takes no untagged arguments");
                }
                Action::Undo
            }
            (_, _, Some(ed)) => {
                if !pos_args.is_empty() {
                    panic!("{:?} takes no untagged arguments", ed);
                }
                Action::Edit(Some(ed), selector)
            }
            _ => {
                if pos_args.is_empty() {
                    Action::Edit(None, selector)
                } else {
                    Action::Remove(pos_args)
                }
            }
        };
        let critical = !matches!(
            &action,
            Action::Edit(None, _) | Action::Edit(Some(Editor::Info), _) | Action::Help(_)
        );
        Self {
            action,
            sandbox,
            overwrite,
            critical,
        }
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
        self.pat.push(pat);
    }

    pub fn add_idx(&mut self, idx: String) {
        self.idx.push(idx);
    }

    pub fn add_time(&mut self, time: String) {
        self.time.push(time);
    }
}

#[derive(Debug)]
struct Once<T> {
    data: Option<T>,
}

impl<T> Once<T> {
    fn new() -> Self {
        Self { data: None }
    }

    fn set(&mut self, new: T) {
        if self.data.is_none() {
            self.data = Some(new);
        } else {
            panic!("Attempt to reset a Once")
        }
    }

    fn into_inner(self) -> Option<T> {
        self.data
    }
}
