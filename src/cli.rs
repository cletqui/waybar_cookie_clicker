use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "waybar_cookie_clicker")]
pub struct Cli {
    /// Path to the save file. Defaults to ~/.local/share/waybar_cookie_clicker/state.json
    #[arg(long, short, value_name = "FILE")]
    pub state: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Show,
    Click,
    /// Output waybar JSON for the Nth slot (buildings + upgrades)
    Slot { index: usize },
    /// Buy whatever is in the Nth slot
    BuySlot { index: usize },
}
