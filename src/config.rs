
use crate::*;

use crate::error::{Error, IntoErrorContext};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GameConfig {
    Flat(PathBuf),
    Wide{dir: PathBuf, sync: Option<bool>}
}

impl GameConfig {
    pub fn directory(&self) -> &Path {
        match self {
            Self::Flat(path) => path,
            Self::Wide { dir, sync: _ } => dir,
        }
    }

    pub fn sync(&self) -> bool {
        match self {
            Self::Flat(_) => true,
            Self::Wide { dir: _, sync } => sync.unwrap_or(true),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub remote: PathBuf,
    pub local_dir: PathBuf,
    #[serde(rename = "game")]
    pub games: HashMap<String, GameConfig>,
}

impl Config {
}

#[derive(Debug, Default)]
pub enum Operation {
    #[default]
    Sync,
    Backup,
    Help,
}

#[derive(Debug)]
pub enum GameSelection {
    All,
    One(String),
}

#[derive(Debug, Default)]
pub struct Arguments {
    pub operation: Option<Operation>,
    pub game: Option<GameSelection>,
    pub command: Option<(String, Vec<String>)>,
}

impl Arguments {
    pub fn parse_args(args: &mut std::env::Args) -> Result<Arguments, Error> {
        let mut arguments = Arguments::default();
        let mut args = args.skip(1).peekable();

        let command = loop {
            let arg = args.next();
            if let Some(arg) = arg {
                match &*arg {
                    "--help" => {
                        arguments.operation = Some(Operation::Help);
                    }
                    "--game" => {
                        let game = args.next().and_then(|s| {
                            if &s[..2] == "--" {
                                None
                            }
                            else {
                                Some(s)
                            }
                        }).ok_or(
                            Error::MissingOption("--game".into())
                        )?;
                        arguments.game = Some(GameSelection::One(game));
                    }
                    "--all" => {
                        arguments.game = Some(GameSelection::All);
                    }
                    "--sync" => {
                        arguments.operation = Some(Operation::Sync);
                    }
                    "--backup" => {
                        arguments.operation = Some(Operation::Backup);
                    }
                    command => {
                        break command.into();
                    }
                }
            } else {
                return Ok(arguments);
            }

        };
        arguments.command = Some((command, args.collect()));
        Ok(arguments)
    }
}

#[cfg(target_os = "linux")]
fn default_config_path() -> String {
    // Path::new(&std::env::var("HOME").unwrap()).join(".config/smsync/config.toml");
    "test/config.toml".to_owned()
}

#[cfg(target_os = "windows")]
fn default_config_path() -> String {
    "%appdata%\\smsync\\config.toml".to_owned()
}

pub fn load_config_file() -> Result<Config, Error> {
    let path = std::env::var("SMSYNC_CONFIG_PATH")
        .unwrap_or(default_config_path());

    Ok(toml::from_str::<Config>(
        &fs::read_to_string(&path).with_context(&path)?
    )?)
}

