mod command;
mod config;
mod select;
mod exec;

fn main() {
    let cmd = match command::Command::argparse() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
    let cfg = config::Config::getenv();
    exec::exec(cmd, cfg);
}
