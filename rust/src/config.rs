use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
    root: PathBuf,
    history: PathBuf,
    lock: PathBuf,
    registry: PathBuf,
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
        let mut cfg = Config {
            history: root.clone(),
            lock: root.clone(),
            registry: root.clone(),
            root,
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
}
