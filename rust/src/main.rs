mod config;

fn main() {
    let cfg = config::Config::argparse(std::env::args());
    println!("{:?}", cfg);
}
