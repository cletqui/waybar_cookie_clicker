mod cli;
mod config;
mod format;
mod game;
mod storage;
mod watch;

use clap::Parser;
use cli::{Cli, Command};
use game::state::GameState;
use game::{display, engine};

fn main() {
    let cli = Cli::parse();
    if let Command::Config(args) = &cli.command {
        print!("{}", config::modules(args));
        return;
    }
    let path = cli
        .state
        .clone()
        .unwrap_or_else(storage::default_state_path);
    if cli.watch {
        let view: &dyn Fn(&GameState) -> String = match &cli.command {
            Command::Show => &display::show,
            Command::Slot { index } => &|s| display::slot(s, *index),
            _ => {
                eprintln!("--watch only works with `show` and `slot`");
                std::process::exit(2);
            }
        };
        let every = std::time::Duration::from_secs(cli.every);
        if let Err(e) = watch::run(&path, every, view)
            && e.kind() != std::io::ErrorKind::BrokenPipe
        {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

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
        Command::Config(_) => unreachable!(),
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
