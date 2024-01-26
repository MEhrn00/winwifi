use clap::{Args, Subcommand};

mod list;
mod scan;

#[derive(Args, Debug)]
pub struct NetworkArguments {
    /// Action to perform for manipulating WiFi networks
    #[command(subcommand)]
    action: NetworkAction,
}

#[derive(Subcommand, Debug)]
enum NetworkAction {
    /// Scan for available WiFi networks
    Scan(scan::ScanSubCommand),

    /// List available Wifi networks
    List(list::ListSubCommand),
}

impl NetworkArguments {
    pub fn handle_subcommand(self) {
        match self.action {
            NetworkAction::Scan(scan_subcommand) => scan_subcommand.run(),
            NetworkAction::List(list_subcommand) => list_subcommand.run(),
        }
    }
}
