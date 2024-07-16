use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("no command specified")]
    NoCommandSpecifiedError,
    #[error("invalid command specified")]
    InvalidCommandError,
    #[error("missing argument for command")]
    MissingArgument(Command, String),
}

#[derive(Debug, Clone)]
pub enum Command {
    Init,
    Clear,
    Store(String, String),
    Get(String),
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = CliError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "init" => Ok(Self::Init),
            "clear" => Ok(Self::Clear),
            "store" => Ok(Self::Store("".to_string(), "".to_string())),
            "get" => Ok(Self::Get("".to_string())),
            _ => Err(CliError::InvalidCommandError),
        }
    }
}

impl Command {
    fn parse_extra(self, args: &mut impl Iterator<Item = String>) -> Result<Self, CliError> {
        match self {
            Self::Store(_, _) => Ok(Self::Store(
                args.next().ok_or_else(|| {
                    CliError::MissingArgument(self.clone(), "key: string, position: 1".to_string())
                })?,
                args.next().ok_or(CliError::MissingArgument(
                    self,
                    "value: string, position: 2".to_string(),
                ))?,
            )),
            Self::Get(_) => Ok(Self::Get(args.next().ok_or(CliError::MissingArgument(
                self,
                "key: string, position: 1".to_string(),
            ))?)),
            _ => Ok(self),
        }
    }
}

#[derive(Debug, Clone)]
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
        let command = command.parse_extra(args)?;

        Ok(Config { _path, command })
    }
}
