mod command;
mod config;
mod select;

fn main() {
    let cmd = command::Command::argparse();
    let cfg = config::Config::getenv();
    println!("{:?}", cmd);
    println!("{:?}", cfg);
}
