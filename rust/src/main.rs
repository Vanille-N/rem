mod config;

fn main() {
    let cmd = config::Command::argparse(std::env::args());
    println!("{:?}", cmd);
}
