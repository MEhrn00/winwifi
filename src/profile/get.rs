use clap::Args;

#[derive(Args, Debug)]
pub(super) struct GetSubCommand {
    /// Name of the WiFi profile to display info for
    #[arg(short, long)]
    name: String,
}

impl GetSubCommand {
    pub fn run(&self) {
        todo!("winwifi profile get")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn profile_get_subcommand() {}
}
