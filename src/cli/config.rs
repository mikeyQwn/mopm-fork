use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("invalid command specified")]
    InvalidCommandError,
    #[error("missing argument for command `{0:?}`, missing argument: `{1}`")]
    MissingArgument(Command, String),
    #[error("invalid argument specified: `{0}")]
    InvalidArgumentError(String),
}

#[derive(Debug, Clone)]
pub enum Command {
    Init,
    Clear,
    Store(String, String),
    Get(String),
    Shield(String),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Help,
    Version,
}

impl<'a> TryFrom<&'a str> for Argument {
    type Error = CliError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "-v" | "--version" => Self::Version,
            "-h" | "--help" => Self::Help,
            arg => return Err(CliError::InvalidArgumentError(arg.to_string())),
        })
    }
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = CliError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "init" => Ok(Self::Init),
            "clear" => Ok(Self::Clear),
            "store" => Ok(Self::Store("".to_string(), "".to_string())),
            "get" => Ok(Self::Get("".to_string())),
            "shield" => Ok(Self::Shield("".to_string())),
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
            Self::Shield(_) => Ok(Self::Shield(args.next().ok_or(
                CliError::MissingArgument(self, "up | down, position: 1".to_string()),
            )?)),
            _ => Ok(self),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub command: Option<Command>,
    pub show_help: bool,
    pub show_version: bool,
}

impl Config {
    pub fn from_args() -> Result<Self, CliError> {
        let mut args = std::env::args();
        Self::from_iter(&mut args)
    }

    pub fn with_command(mut self, command: Option<Command>) -> Self {
        self.command = command;
        self
    }

    fn from_iter(args: &mut impl Iterator<Item = String>) -> Result<Self, CliError> {
        let mut args = args.skip(1);
        let mut config = Self::default();
        let maybe_command = match args.next() {
            Some(v) => v,
            None => return Ok(config),
        };
        let mut first_arg = vec![];

        match Command::try_from(maybe_command.as_str()) {
            Ok(command) => config = config.with_command(Some(command.parse_extra(&mut args)?)),
            Err(_) if maybe_command.starts_with('-') => first_arg.push(maybe_command),
            Err(err) => return Err(err),
        }

        first_arg
            .into_iter()
            .chain(args)
            .try_fold(config, |acc, v| {
                Ok(acc.apply_argument(Argument::try_from(v.as_str())?))
            })
    }

    pub fn apply_argument(mut self, argument: Argument) -> Self {
        match argument {
            Argument::Version => self.show_version = true,
            Argument::Help => self.show_help = true,
        }
        self
    }
}
