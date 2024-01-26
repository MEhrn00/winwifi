use clap::Args;

#[derive(Args, Debug)]
pub(super) struct ListSubCommand;

impl ListSubCommand {
    pub fn run(&self) {
        todo!("winwifi network list")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn network_list_subcommand() {}
}
