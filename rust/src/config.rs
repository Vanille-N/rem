use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
    root: PathBuf,
    history: PathBuf,
    lock: PathBuf,
    registry: PathBuf,
    ls_cmd: &'static str,
    fzf_cmd: &'static str,
}

impl Config {
    pub fn getenv() -> Self {
        let root = match std::env::var("REM_ROOT") {
            Ok(s) => PathBuf::from(s),
            Err(_) => match std::env::var("HOME") {
                Ok(s) => {
                    let mut path = PathBuf::from(s);
                    path.push("._trash");
                    path
                }
                Err(_) => PathBuf::from("/tmp/trash"),
            },
        };
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let ls_cmd = get_ls_cmd();
        let fzf_cmd = get_fzf_cmd();
        let mut cfg = Config {
            history: root.clone(),
            lock: root.clone(),
            registry: root.clone(),
            root,
            ls_cmd,
            fzf_cmd,
        };
        cfg.history.push("history");
        cfg.lock.push("lock");
        cfg.registry.push("registry");
        std::fs::create_dir_all(&cfg.registry).unwrap();
        cfg
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn history(&self) -> &Path {
        self.history.as_path()
    }

    pub fn registry(&self) -> &Path {
        self.registry.as_path()
    }

    pub fn fzf_cmd(&self) -> &'static str {
        self.fzf_cmd
    }

    pub fn ls_cmd(&self) -> &'static str {
        self.ls_cmd
    }
}

fn cmd_exists(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--fail")
        .output()
        .is_ok()
}

fn default_ls_cmd() -> &'static str {
    if cmd_exists("exa") {
        "exa"
    } else {
        "ls"
    }
}

fn get_ls_cmd() -> &'static str {
    match std::env::var("REM_LS").ok().as_deref() {
        Some("ls") => "ls",
        Some("exa") if cmd_exists("exa") => {
            "exa"
        }
        Some(other) => {
            let err = crate::command::Error::InvalidVarLs(other.to_string());
            eprintln!("{}", err);
            default_ls_cmd()
        }
        None => default_ls_cmd(),
    }
}

fn default_fzf_cmd() -> &'static str {
    if cmd_exists("sk") {
        "sk"
    } else if cmd_exists("fzf") {
        "fzf"
    } else {
        let err = crate::command::Error::NoInstalledFzf;
        println!("{}", err);
        "sk"
    }
}

fn get_fzf_cmd() -> &'static str {
    match std::env::var("REM_FZF").ok().as_deref() {
        Some("fzf") if cmd_exists("fzf") => "fzf",
        Some("sk") if cmd_exists("sk") => "sk",
        Some(other) => {
            let err = crate::command::Error::InvalidVarFzf(other.to_string());
            eprintln!("{}", err);
            default_fzf_cmd()
        }
        None => default_fzf_cmd(),
    }
}
