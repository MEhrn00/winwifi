use clap::Args;

#[derive(Args, Debug)]
pub(super) struct RemoveSubCommand {}

impl RemoveSubCommand {
    pub fn run(&self) {
        todo!("winwifi profile remove")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn profile_remove_subcommand() {}
}
