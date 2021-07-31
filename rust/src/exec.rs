use crate::{
    command::{self, Action, Command, Error},
    config::Config,
    select::Entry,
};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn exec(cmd: Command, cfg: Config) {
    match cmd.action {
        Action::Remove(files) => {
            let mut register = Vec::new();
            for f in files {
                match remove(&cfg, cmd.sandbox, f) {
                    Ok(entry) => register.push(entry),
                    Err(Error::SandBoxed) => (),
                    Err(err) => eprintln!("{}", err),
                }
            }
            use std::io::Write;
            let mut history = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(cfg.history())
                .unwrap();
            writeln!(history, "").unwrap_or_else(|_| {
                let err = Error::FailedToWrite(
                    cfg.registry().to_str().unwrap().to_string(),
                    Some("".to_string()),
                );
                eprintln!("{}", err)
            });
            for entry in register {
                let contents = format!("{}|{}|{}", entry.alias, entry.name, entry.timestamp);
                writeln!(history, "{}", contents).unwrap_or_else(|_| {
                    let err = Error::FailedToWrite(
                        cfg.registry().to_str().unwrap().to_string(),
                        Some(contents),
                    );
                    eprintln!("{}", err)
                });
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
    let randname = generate_random_dirname();
    let alias = {
        let mut p = PathBuf::new();
        p.push(&randname);
        p
    };
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut destination = cfg.registry().to_path_buf();
    destination.push(&alias);
    let mut destfile = destination.clone();
    let mut destdata = destination.clone();
    destfile.push(Path::new("file"));
    destdata.push(Path::new("meta"));
    if sandbox {
        println!("Create directory '{}'", destination.to_str().unwrap());
        println!("Register data as '{}'", destdata.to_str().unwrap());
        println!(
            "Move '{}' to '{}'",
            path.to_str().unwrap(),
            destfile.to_str().unwrap()
        );
        //println!("Save '{}|{}|{}' into '{}'", alias.to_str().unwrap(), path.to_str().unwrap(), timestamp, destdata.to_str().unwrap());
    } else {
        std::fs::create_dir(&destination).or_else(|_| {
            Err(Error::CouldNotCreateDir(
                destination.to_str().unwrap().to_string(),
            ))
        })?;
        record_data(cfg, &path, &destdata)?;
        std::fs::rename(&path, &destfile).or_else(|_| {
            std::fs::remove_dir_all(&destination).unwrap();
            Err(Error::CouldNotMove(
                path.to_str().unwrap().to_string(),
                destfile.to_str().unwrap().to_string(),
            ))
        })?;
    }
    Ok(Entry {
        name: path.to_str().unwrap().to_string(),
        alias: randname,
        timestamp,
    })
}

fn record_data(cfg: &Config, file: &Path, meta: &Path) -> Result<(), Error> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(meta)
        .unwrap();
    let date_out = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .or_else(|_| Err(Error::ExecError("date")))?;
    let ls_out = std::process::Command::new(cfg.ls_cmd())
        .arg("-Flah")
        .arg("--color=always")
        .arg(file.to_str().unwrap())
        .output()
        .or_else(|_| Err(Error::ExecError(cfg.ls_cmd())))?;
    let file_out = std::process::Command::new("file")
        .arg(file.to_str().unwrap())
        .output()
        .or_else(|_| Err(Error::ExecError("file")))?;
    use std::io::Write;
    writeln!(f, "{}", file.to_str().unwrap())
        .and_then(|_| writeln!(f, "{}", std::str::from_utf8(&date_out.stdout).unwrap()))
        .and_then(|_| writeln!(f, "\n{}", std::str::from_utf8(&ls_out.stdout).unwrap()))
        .and_then(|_| writeln!(f, "\n{}", std::str::from_utf8(&file_out.stdout).unwrap()))
        .or_else(|_| {
            Err(Error::FailedToWrite(
                file.to_str().unwrap().to_string(),
                None,
            ))
        })
}

const ALIAS_LENGTH: usize = 25;

fn generate_random_dirname() -> String {
    use rand::{Rng, distributions::Alphanumeric};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(ALIAS_LENGTH)
        .map(char::from)
        .collect()
}
