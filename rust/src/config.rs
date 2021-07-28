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
                    path.push(".trash");
                    path
                }
                Err(_) => PathBuf::from("/tmp/trash"),
            },
        };
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

fn get_ls_cmd() -> &'static str {
    unimplemented!()
}

fn get_fzf_cmd() -> &'static str {
    unimplemented!()
}
