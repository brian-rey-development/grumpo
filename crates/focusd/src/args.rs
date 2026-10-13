use std::ffi::OsString;
use std::path::PathBuf;

use thiserror::Error;

pub const USAGE: &str = "usage:
  focusd run --owner-uid <uid>        serve the Grumpo socket until stopped
  focusd import-legacy <script-path>  create the state from the old focus-hosts script
  focusd strip-hosts                  remove the managed block from the hosts file";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run {
        owner_uid: u32,
    },
    ImportLegacy {
        script: PathBuf,
    },
    StripHosts,
    /// Not in the usage text, because only the installer needs it.
    Ping,
}

const ROOT_UID: u32 = 0;
const NO_USER_UID: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArgsError {
    #[error("no command given")]
    MissingCommand,
    #[error("unknown command \"{0}\"")]
    UnknownCommand(String),
    #[error("missing {0}")]
    Missing(&'static str),
    #[error("\"{0}\" is not a valid user id")]
    InvalidUid(String),
    #[error(
        "--owner-uid 0 would hand the socket to root, pass the id of the user who runs Grumpo"
    )]
    RootUid,
    #[error(
        "--owner-uid {NO_USER_UID} is not a user, chown reads it as \"leave the owner alone\""
    )]
    NoUserUid,
    #[error("unexpected argument \"{0}\"")]
    Unexpected(String),
}

/// Parses the arguments after the program name.
pub fn parse<I: IntoIterator<Item = OsString>>(args: I) -> Result<Command, ArgsError> {
    let mut args = args.into_iter();
    let name = args.next().ok_or(ArgsError::MissingCommand)?;
    let command = match name.to_str() {
        Some("run") => run(&mut args)?,
        Some("import-legacy") => import_legacy(&mut args)?,
        Some("strip-hosts") => Command::StripHosts,
        Some("ping") => Command::Ping,
        _ => {
            return Err(ArgsError::UnknownCommand(
                name.to_string_lossy().into_owned(),
            ))
        }
    };
    match args.next() {
        Some(extra) => Err(ArgsError::Unexpected(extra.to_string_lossy().into_owned())),
        None => Ok(command),
    }
}

fn run(args: &mut impl Iterator<Item = OsString>) -> Result<Command, ArgsError> {
    match args.next() {
        Some(flag) if flag == "--owner-uid" => {}
        Some(other) => return Err(ArgsError::Unexpected(other.to_string_lossy().into_owned())),
        None => return Err(ArgsError::Missing("--owner-uid <uid>")),
    }
    let value = args
        .next()
        .ok_or(ArgsError::Missing("a value for --owner-uid"))?;
    let text = value.to_string_lossy();
    let owner_uid = text
        .parse()
        .map_err(|_| ArgsError::InvalidUid(text.into_owned()))?;
    match owner_uid {
        ROOT_UID => Err(ArgsError::RootUid),
        NO_USER_UID => Err(ArgsError::NoUserUid),
        owner_uid => Ok(Command::Run { owner_uid }),
    }
}

fn import_legacy(args: &mut impl Iterator<Item = OsString>) -> Result<Command, ArgsError> {
    let script = args.next().ok_or(ArgsError::Missing("<script-path>"))?;
    Ok(Command::ImportLegacy {
        script: PathBuf::from(script),
    })
}

#[cfg(test)]
mod tests;
