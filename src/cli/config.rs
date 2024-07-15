use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("no command specified")]
    NoCommandSpecifiedError,
    #[error("invalid command specified")]
    InvalidCommandError,
}

pub enum Command {
    Init,
    Clear,
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = CliError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "init" => Ok(Self::Init),
            "clear" => Ok(Self::Clear),
            _ => Err(CliError::InvalidCommandError),
        }
    }
}

pub struct Config {
    pub _path: String,
    pub command: Command,
}

impl Config {
    pub fn from_args() -> Result<Self, CliError> {
        let mut args = std::env::args();
        Self::from_iter(&mut args)
    }

    fn from_iter(args: &mut impl Iterator<Item = String>) -> Result<Self, CliError> {
        let _path = args.next().ok_or(CliError::NoCommandSpecifiedError)?;
        let command_str: String = args.next().ok_or(CliError::NoCommandSpecifiedError)?;

        let command: Command = command_str.as_str().try_into()?;

        Ok(Config { _path, command })
    }
}
