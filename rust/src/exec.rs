use crate::{
    command::{self, Action, Command, Error},
    config::Config,
    select::{Entry, Select},
};
use std::fmt;
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
        Action::Edit(ed, sel) => {
            let entries = match crate::select::Entries::from_file(cfg.history()) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            };
            let mut selection = std::collections::BTreeSet::new();
            match sel.make() {
                Err(e) => eprintln!("{}", e),
                Ok(sel) => {
                    sel.select(&entries, &mut selection);
                    ed.run(&cfg, &entries, &selection);
                }
            }
        }
        Action::Help(menus) => {
            if menus.is_empty() {
                println!("{}", MSG_HELP_MAIN);
            } else {
                for menu in menus {
                    println!(
                        "{}",
                        match menu.as_str() {
                            "main" => MSG_HELP_MAIN,
                            "examples" => MSG_HELP_EXAMPLES,
                            "cmd" => MSG_HELP_CMD,
                            "info" => MSG_HELP_INFO,
                            "rest" => MSG_HELP_REST,
                            "undo" => MSG_HELP_UNDO,
                            "del" => MSG_HELP_DEL,
                            "select" => MSG_HELP_SELECT,
                            "pat" => MSG_HELP_PAT,
                            "fzf" => MSG_HELP_FZF,
                            "blk" => MSG_HELP_BLK,
                            "time" => MSG_HELP_TIME,
                            "intro" => MSG_HELP_INTRO,
                            "config" => MSG_HELP_CONFIG,
                            other => {
                                eprintln!("{}", Error::HelpNotFound(other.to_string()));
                                continue;
                            }
                        }
                    );
                }
            }
        }
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
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(ALIAS_LENGTH)
        .map(char::from)
        .collect()
}

const MSG_HELP_MAIN: &str = include_str!("../../help/main.ansi");
const MSG_HELP_EXAMPLES: &str = include_str!("../../help/examples.ansi");
const MSG_HELP_CMD: &str = include_str!("../../help/cmd.ansi");
const MSG_HELP_INFO: &str = include_str!("../../help/info.ansi");
const MSG_HELP_REST: &str = include_str!("../../help/rest.ansi");
const MSG_HELP_UNDO: &str = include_str!("../../help/undo.ansi");
const MSG_HELP_DEL: &str = include_str!("../../help/del.ansi");
const MSG_HELP_SELECT: &str = include_str!("../../help/select.ansi");
const MSG_HELP_PAT: &str = include_str!("../../help/pat.ansi");
const MSG_HELP_FZF: &str = include_str!("../../help/fzf.ansi");
const MSG_HELP_BLK: &str = include_str!("../../help/blk.ansi");
const MSG_HELP_TIME: &str = include_str!("../../help/time.ansi");
const MSG_HELP_INTRO: &str = include_str!("../../help/intro.ansi");
const MSG_HELP_CONFIG: &str = include_str!("../../help/config.ansi");
