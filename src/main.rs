extern crate clap;

mod config;

fn main() {
    let config = config::parse_args();
    println!("Hello, world! {}", config.config_file);
}
