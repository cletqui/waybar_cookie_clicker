use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::ConfigArgs;

#[derive(Parser)]
#[command(name = "waybar_cookie_clicker")]
pub struct Cli {
    /// Path to the save file. Defaults to ~/.local/share/waybar_cookie_clicker/state.json
    #[arg(long, short, value_name = "FILE")]
    pub state: Option<PathBuf>,

    /// Stay running and print a line whenever the output changes (for `show` and `slot`)
    #[arg(long)]
    pub watch: bool,

    /// Seconds between refreshes in --watch mode
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..))]
    pub every: u64,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Show,
    Click,
    /// Output waybar JSON for the Nth slot (buildings + upgrades)
    Slot {
        index: usize,
    },
    /// Buy whatever is in the Nth slot
    BuySlot {
        index: usize,
    },
    /// Wipe all progress and start over
    Reset,
    /// Print waybar module definitions (JSON) for the given layout
    Config(ConfigArgs),
}
