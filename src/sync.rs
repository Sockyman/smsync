
use std::{
    path::{Path, PathBuf},
    fs,
    str::FromStr,
    process,
};
use crate::{
    error::{Error, IntoErrorContext},
    hashsum::{self, HashSum},
    config::Config,
};
use log::error;
use chrono::{
    Local,
    DateTime,
};

#[derive(Clone, PartialEq, Eq)]
enum ToSync {
    NoSync,
    Cloud,
    Local,
}

#[derive(PartialEq, Eq)]
pub enum SyncResult {
    Continue,
    Abort,
}

pub fn backup(config: &Config, game: &str) -> Result<(), Error> {
    let _config = config
        .games
        .get(game)
        .ok_or(Error::InvalidGame(game.to_owned()))?;
    unimplemented!()
}

fn path_datetime(path: impl AsRef<Path>) -> Result<DateTime<Local>, Error> {
    Ok(
        fs::metadata(&path)
            .with_context(&path)?
            .modified()
            .with_context(&path)?
            .into()
    )
}

fn backup_filename(backup_dir: &Path, prefix: &str) -> PathBuf {
    let datetime = Local::now();
    let mut path = prefix.to_owned();
    path.push_str(&datetime.to_rfc3339());
    backup_dir.join(&path)
}

fn syncronize_directories(
    from: &Path,
    to: &Path,
    backup: &Path
) -> Result<(), Error> {
    println!("syncing: {:?} -> {:?}", from, to);
    fs::create_dir_all(&backup).with_context(backup)?;
    fs::rename(to, backup_filename(backup, "implicit_")).with_context(backup)?;
    fs::create_dir(to).with_context(to)?;
    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.content_only = true;
    fs_extra::dir::copy(from, to, &copy_options).unwrap();
    Ok(())
}

fn get_tosync(
    lastsync_time: DateTime<Local>,
    has_abort_option: bool
) -> Option<ToSync> {
    let mut options = vec![("keep local".into(), Some(ToSync::Local)),
        ("keep remote".into(), Some(ToSync::Cloud)),
        ("ignore".into(), Some(ToSync::NoSync)),
    ];

    if has_abort_option {
        options.push(("abort".into(), None));
    }

    crate::ui::show_question("Conflict".into(),
        format!("{}\n\n{} {}\n\n{}",
            "Both the remote and local saves have updated sinc last sync.",
            "last synced",
            lastsync_time,
            "Which version should be kept?"
        ),
        options,
        None
    )
}

fn sync_conflict_resolver(
    config: &Config,
    game: &str,
    conflict_resolver: impl FnOnce(DateTime<Local>) -> Option<ToSync>
) -> Result<SyncResult, Error> {
    let game_config = config
        .games
        .get(game)
        .ok_or(Error::InvalidGame(game.to_owned()))?;

    if !game_config.sync() {
        // error
    }

    let remote_path = config.remote.join(game);
    let local_path = config.local_dir.join(game);

    let head_path = remote_path.join("head");
    let lastsync_path = local_path.join("lastsync");

    let local_hash = hashsum::hash_directory(game_config.directory())?;

    fs::create_dir_all(&head_path).with_context(&head_path)?;
    let remote_hash = hashsum::hash_directory(&head_path)?;

    fs::create_dir_all(&local_path).with_context(&local_path)?;

    let lastsync_hash = match fs::read_to_string(&lastsync_path) {
        Ok(val) => HashSum::from_str(&val)?,
        Err(_) => HashSum::default(),
    };

    let local_ahead = local_hash != lastsync_hash;
    let remote_ahead = remote_hash != lastsync_hash;
    let remote_differs = local_hash != remote_hash;

    println!(
        "game: {}\nsync: {}\nhead: {}",
        &local_hash.to_string()[..16],
        &lastsync_hash.to_string()[..16],
        &remote_hash.to_string()[..16]
    );

    let to_sync = if !remote_differs {
        ToSync::NoSync
    } else if local_ahead && remote_ahead {
        let lastsync_time = path_datetime(&lastsync_path).unwrap_or_default();

        match conflict_resolver(lastsync_time) {
            Some(value) => value,
            None => return Ok(SyncResult::Abort),
        }
    } else if local_ahead {
        ToSync::Local
    } else if remote_ahead {
        ToSync::Cloud
    } else {
        ToSync::NoSync
    };

    match to_sync {
        ToSync::Local => {
            let backup = remote_path.join("backup");
            syncronize_directories(
                game_config.directory(),
                &head_path,
                &backup
            )?;
            fs::write(&lastsync_path, local_hash.to_string())
                .with_context(lastsync_path)?;
        }
        ToSync::Cloud => {
            let backup = local_path.join("backup");
            syncronize_directories(
                &head_path,
                game_config.directory(),
                &backup
            )?;
            fs::write(&lastsync_path, remote_hash.to_string())
                .with_context(lastsync_path)?;
        }
        ToSync::NoSync => {}
    }

    Ok(SyncResult::Continue)
}

pub fn sync(config: &Config, game: &str) -> Result<SyncResult, Error> {
    sync_conflict_resolver(&config, &game, |dt| get_tosync(dt, false))
}

pub fn sync_run(
    config: &Config,
    game: &str,
    command: &str,
    params: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>
) -> Result<(), Error> {
    let sync_result = sync_conflict_resolver(
        &config,
        &game,
        |dt| get_tosync(dt, true)
    )?;

    if sync_result == SyncResult::Continue {
        let mut c = process::Command::new(&command);
        c.args(params);
        let mut p = c.spawn().with_context(command)?;
        let e = p.wait().with_context(command)?;
        if !e.success() {
            error!(
                "'{}' exited with status {}",
                &command,
                e.code().unwrap_or(0)
            );
        }
        sync(&config, &game)?;
    }
    Ok(())
}

