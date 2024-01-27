use clap::Args;

#[derive(Args, Debug)]
pub(super) struct ScanSubCommand;

impl ScanSubCommand {
    pub fn run(&self) {
        todo!("winwifi network scan")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn network_scan_subcommand() {}
}
