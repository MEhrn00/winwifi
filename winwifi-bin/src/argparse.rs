use crate::{network, profile};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(super) struct ProgramArguments {
    /// Action to perform
    #[command(subcommand)]
    command: ProgramCommand,
}

#[derive(Subcommand, Debug)]
pub(super) enum ProgramCommand {
    /// Manipulate WiFi profiles
    Profile(profile::ProfileArguments),

    /// Manage connections to WiFi networks
    Network(network::NetworkArguments),
}

impl ProgramArguments {
    pub fn handle_arguments(self) {
        match self.command {
            ProgramCommand::Profile(profile_arguments) => profile_arguments.handle_subcommand(),
            ProgramCommand::Network(network_arguments) => network_arguments.handle_subcommand(),
        }
    }
}
