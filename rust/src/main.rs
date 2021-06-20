mod command;
mod config;

fn main() {
    let cmd = command::Command::argparse();
    let cfg = config::Config::getenv();
    println!("{:?}", cmd);
    println!("{:?}", cfg);
}
