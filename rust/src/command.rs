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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Default)]
pub struct Selector {
    pat: PatternList,
    idx: IndexList,
    time: TimeList,
    fzf: bool,
}

#[derive(Debug)]
pub enum Error {
    NonExclusiveCmd(&'static str, &'static str),
    TooManyArgs(&'static str),
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
    pub fn argparse() -> Result<Self, Error> {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse<I>(args: I) -> Result<Self, Error>
    where I: Iterator<Item = String> {
        let mut pos_args = Vec::new();
        let mut selector = Selector::default();
        let mut help = false;
        let mut undo = false;
        let mut editor = OnceEd::new();
        let mut sandbox = false;
        let mut overwrite = false;
        let mut args = args.peekable();
        loop {
            match args.next() {
                None => break,
                Some(arg) => match arg.as_str() {
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
            (true, true, _) => return Err(Error::NonExclusiveCmd("help", "undo")),
            (true, _, Some(ed)) => return Err(Error::NonExclusiveCmd("help", ed.as_str())),
            (_, true, Some(ed)) => return Err(Error::NonExclusiveCmd("undo", ed.as_str())),
            // Ok
            (true, _, _) => Action::Help(pos_args),
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
                    Action::Remove(pos_args)
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
            Err(Error::NonExclusiveCmd(self.data.unwrap().as_str(), new.as_str()))
        }
    }

    fn into_inner(self) -> Option<Editor> {
        self.data
    }
}
