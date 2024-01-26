use clap::Args;

#[derive(Args, Debug)]
pub(super) struct ListSubCommand;

impl ListSubCommand {
    pub fn run(&self) {
        todo!("winwifi profile list")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn profile_list_subcommand() {}
}
