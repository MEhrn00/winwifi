use clap::Parser;

mod argparse;
mod network;
mod profile;

fn main() {
    let program = argparse::ProgramArguments::parse();
    program.handle_arguments();
}
