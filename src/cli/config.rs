use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("invalid command specified")]
    InvalidCommandError,
    #[error("missing argument for command `{0:?}`, missing argument: `{1}`")]
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
    pub command: Option<Command>,
}

impl Config {
    pub fn from_args() -> Result<Self, CliError> {
        let mut args = std::env::args();
        Self::from_iter(&mut args)
    }

    fn from_iter(args: &mut impl Iterator<Item = String>) -> Result<Self, CliError> {
        let command: Option<Command> = args
            .skip(1)
            .next()
            .map(|cmd_str| {
                cmd_str
                    .as_str()
                    .try_into()
                    .and_then(|cmd: Command| cmd.parse_extra(args))
            })
            .transpose()?;

        Ok(Config { command })
    }
}
