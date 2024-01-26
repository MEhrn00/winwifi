use clap::Parser;

mod argparse;
mod network;
mod profile;

fn main() {
    let winwifi = argparse::ProgramArguments::parse();
    winwifi.handle_arguments();
}
