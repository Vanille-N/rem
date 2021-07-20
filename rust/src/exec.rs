use crate::{
    command::{self, Command, Action, Error},
    config::Config,
    select::Entry,
};
use std::time::SystemTime;
use std::path::{Path, PathBuf};

pub fn exec(cmd: Command, cfg: Config) {
    match cmd.action {
        Action::Remove(files) => {
            let mut register = Vec::new();
            for f in files {
                match remove(&cfg, cmd.sandbox, f) {
                    Ok(entry) => register.push(entry),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        Action::Edit(ed, sel) => unimplemented!(),
        Action::Undo => unimplemented!(),
        Action::Help(menus) => unimplemented!(),
    }
}

fn remove(cfg: &Config, sandbox: bool, file: command::File) -> Result<Entry, Error> {
    let mut path = std::env::current_dir().unwrap();
    path.push(file.make());
    let path = match path.canonicalize() {
        Ok(name) => path,
        Err(_) => return Err(Error::FileDoesNotExist(file.contents())),
    };
    let alias = Path::new("WXYZ");
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let mut destination = cfg.registry().to_path_buf();
    destination.push(&alias);
    let mut destfile = destination.clone();
    let mut destdata = destination.clone();
    destfile.push(Path::new("file"));
    destdata.push(Path::new("meta"));
    if sandbox {
        println!("Create directory '{}'", destination.to_str().unwrap());
        println!("Register data as '{}'", destdata.to_str().unwrap());
        println!("Move '{}' to '{}'", path.to_str().unwrap(), destfile.to_str().unwrap());
        println!("Save '{}|{}|{}' into '{}'", alias.to_str().unwrap(), path.to_str().unwrap(), timestamp, destdata.to_str().unwrap());
        Err(Error::SandBoxed)
    } else {
        unimplemented!()
    }
}
