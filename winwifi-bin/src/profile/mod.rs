use clap::{Args, Subcommand};

mod get;
mod list;
mod remove;

#[derive(Args, Debug)]
pub struct ProfileArguments {
    /// Action to perform for manipulating WiFi profiles
    #[command(subcommand)]
    action: ProfileAction,
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    /// List saved WiFi profiles
    List(list::ListSubCommand),

    /// Get information about a saved WiFi profile
    Get(get::GetSubCommand),

    /// Remove a saved WiFi profile
    Remove(remove::RemoveSubCommand),
}

impl ProfileArguments {
    pub fn handle_subcommand(self) {
        match self.action {
            ProfileAction::List(list_subcommand) => list_subcommand.run(),
            ProfileAction::Get(get_subcommand) => get_subcommand.run(),
            ProfileAction::Remove(remove_subcommand) => remove_subcommand.run(),
        }
    }
}
