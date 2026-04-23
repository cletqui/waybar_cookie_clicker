mod cli;
mod format;
mod game;
mod storage;

use clap::Parser;
use cli::{Cli, Command};
use game::{display, engine};

fn main() {
    let cli = Cli::parse();
    let path = cli
        .state
        .clone()
        .unwrap_or_else(storage::default_state_path);
    let mut state = storage::load(&path);

    engine::tick(&mut state);

    let (output, dirty) = match cli.command {
        Command::Show => (display::show(&state), false),
        Command::Click => {
            engine::click(&mut state);
            (display::show(&state), true)
        }
        Command::Slot { index } => (display::slot(&state, index), false),
        Command::BuySlot { index } => {
            engine::buy_slot(&mut state, index);
            (display::show(&state), true)
        }
        Command::Reset => {
            state = Default::default();
            (display::show(&state), true)
        }
    };

    print!("{output}");
    if dirty {
        storage::save(&path, &state);
    }
}
