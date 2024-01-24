#![allow(dead_code)]

mod hashsum;
mod error;
mod config;
mod sync;
mod ui;

use std::{
    fs,
    path::{Path, PathBuf},
    collections::HashMap,
    process,
};
use serde::Deserialize;
use config::{
    Arguments,
    Config,
    GameSelection,
    Operation
};
use error::Error;

const PROGRAM_NAME: &str = "smsync";

const HELP_MESSAGE: &str = "Usage:
 smsync (--game <gameid> | --all) (--sync | --backup)
 smsync --game <gameid> --sync <command> [arguments...]

Options:
 --sync
 --game <gameid>
 --all
 --backup
 --help";


fn operation_sync(
    config: Config,
    game: Option<GameSelection>,
    command: Option<(String, Vec<String>)>
) -> Result<(), Error> {
    match (game, command) {
        (Some(GameSelection::One(game)), Some((command, params))) => {
            sync::sync_run(&config, &game, &command, &params)?;
        }
        (Some(GameSelection::One(game)), None) => {
            sync::sync(&config, &game)?;
        }
        (Some(GameSelection::All), None) => {
            for (game, _) in config.games.iter() {
                sync::sync(&config, game)?;
            }
        }
        _ => {
            return Err(Error::InvalidArgument(
                "incorrect options to --sync".into()
            ));
        }
    }
    Ok(())
}

fn operation_backup(
    config: Config,
    game: Option<GameSelection>,
    command: Option<(String, Vec<String>)>
) -> Result<(), Error> {
    match (game, command) {
        (Some(GameSelection::One(game)), None) => {
            sync::backup(&config, &game).unwrap();
        }
        (Some(GameSelection::All), None) => {
            for (game, _) in config.games.iter() {
                sync::backup(&config, game).unwrap();
            }
        }
        (_, None) => {
            return Err(Error::InvalidArgument(
                "command not supported for --backup".into()
            ));
        }
        _ => {
            return Err(Error::InvalidArgument(
                "incorrect options to --backup".into()
            ));
        }

    }
    Ok(())
}

fn run() -> Result<(), Error> {
    let config = config::load_config_file()?;
    let arguments = Arguments::parse_args(&mut std::env::args());

    if let Ok(arguments) = arguments {
        match arguments.operation {
            Some(Operation::Sync) => {
                operation_sync(config, arguments.game, arguments.command)
            }
            Some(Operation::Backup) => {
                operation_backup(config, arguments.game, arguments.command)
            }
            Some(Operation::Help) => {
                println!("{}", HELP_MESSAGE);
                Ok(())
            }
            None if arguments.game.is_none() && arguments.command.is_none() => {
                println!("{}", HELP_MESSAGE);
                Ok(())
            }
            None => {
                Err(Error::InvalidArgument("missing operation".into()))
            }
        }?;
    };

    Ok(())
}

fn main() -> std::process::ExitCode {
    env_logger::init();

    if let Err(err) = run() {
        ui::show_error(&err);
        process::ExitCode::FAILURE
    } else {
        process::ExitCode::SUCCESS
    }
}

