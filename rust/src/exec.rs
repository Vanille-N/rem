use crate::{
    command::{self, Command, Action, Error},
    config::Config,
    select::Entry,
};
use std::time::SystemTime;

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
    let path = file.make();
    let path = match path.canonicalize() {
        Ok(name) => path,
        Err(_) => return Err(Error::FileDoesNotExist(file.contents())),
    };
    let alias = Path::from("WXYZ");
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let destination = cfg.registry().clone();
    if sandbox {
        println!("Create directory '{}'");
        println!("Register data as '{}'");
        println!("Move '{}' to '{}'");
        println!("Save '{}|{}|{}' into '{}'");
    }
    unimplemented!()
}
