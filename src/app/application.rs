use inotify::{Inotify, WatchMask};

use crate::{
    cli::{
        config::{Command, Config},
        terminal::Terminal,
    },
    core::{
        encoder::{Encoder, EncoderError},
        encoding::version::Version,
        encryptor::{DynamicEncryptor, Encryprtor},
        identifiers::Identifiable,
        manager::PasswordManager,
    },
    log::logger::Logger,
    storage::store::{Storage, StorageError},
};

use super::constants;

pub struct App<T>
where
    T: term::Terminal,
{
    config: Config,
    logger: Logger<T>,
}

impl<T> App<T>
where
    T: term::Terminal,
{
    pub fn new(config: Config, logger: Logger<T>) -> Self {
        App { config, logger }
    }

    pub fn run(&mut self) {
        if self.handle_breaking_arguments() {
            return;
        }

        let command = match self.config.command.take() {
            None => {
                self.logger.info(constants::NO_COMMAND_SPECIFIED.as_ref());
                return;
            }
            Some(v) => v,
        };

        match command {
            Command::Init => self.handle_init(),
            Command::Clear => self.handle_clear(),
            Command::Store(key, value) => {
                self.with_init(|app| app.handle_store(key.as_ref(), value.as_ref()))
            }
            Command::Get(key) => self.with_init(|app| app.handle_get(key.as_ref())),

            Command::Shield(v) => match v.as_str() {
                "up" => self.with_init(|app| app.handle_shield_up()),
                "down" => self.handle_shield_down(),
                _ => self
                    .logger
                    .fatal("invalid argument, accepted: `up`, `down`".as_ref()),
            },
        }
    }

    fn handle_breaking_arguments(&mut self) -> bool {
        if self.config.show_version {
            self.logger
                .info(format!("Version: {}\n", Version::current_version()).as_ref());
            return true;
        }
        if self.config.show_help {
            self.logger.info(constants::HELP_MESSAGE.as_ref());
            return true;
        }
        false
    }

    fn handle_init(&mut self) {
        if Storage::is_initialized().unwrap() {
            self.logger.warn(constants::ALREADY_INITIALIZED.as_ref());
            return;
        }

        let password = self.prompt_password();
        let mut pm = PasswordManager::init(password.trim());

        match Storage::init(&mut pm) {
            Ok(_) => self.logger.info(constants::INIT_SUCCESSFULL.as_ref()),
            Err(StorageError::RootAlreadyExistsErorr) => {}
            Err(err) => self.logger.fatal(err.to_string().as_ref()),
        }
    }

    fn handle_clear(&mut self) {
        match Storage::clear() {
            Ok(_) => {
                self.logger.info(constants::CLEAR_SUCCESSFUL.as_ref());
            }
            Err(StorageError::RootDoesNotExistErorr) => {
                self.logger.info(constants::NOT_INITIALIZED.as_ref());
            }
            Err(err) => self.logger.fatal(err.to_string().as_ref()),
        }
    }

    fn handle_store(&mut self, key: &str, value: &str) {
        let mut pm = self.get_password_manager();
        pm.store_password(key.into(), value).unwrap();
        if let Err(err) = self.save_password_manager(&mut pm) {
            self.logger.error(&err);
            self.logger.fatal(constants::ERROR_WHILE_SAVING.as_ref())
        };
        self.logger.info(constants::STORE_SUCCESSFUL.as_ref());
    }

    fn handle_get(&mut self, key: &str) {
        let mut pm = self.get_password_manager();
        self.logger.info(pm.get_password(key).unwrap().as_ref());
    }

    fn prompt_password(&mut self) -> String {
        self.logger.info(constants::PASSWORD_PROMPT.as_ref());
        self.logger.flush();
        Terminal::read_password()
    }

    fn get_password_manager(&mut self) -> PasswordManager<DynamicEncryptor> {
        let password = self.prompt_password();
        let mut pm_reader = match Storage::get_data_reader() {
            Ok(v) => v,
            Err(err) => self.logger.fatal(err.to_string().as_ref()),
        };
        match Encoder::decode(password.trim().as_ref(), &mut pm_reader) {
            Ok(v) => v,
            Err(err) => self.logger.fatal(err.to_string().as_ref()),
        }
    }

    fn save_password_manager<U>(
        &self,
        password_manager: &mut PasswordManager<U>,
    ) -> Result<(), EncoderError>
    where
        U: Encryprtor + Identifiable,
    {
        let mut writer = Storage::get_data_writer().unwrap();
        Encoder::encode(&mut writer, password_manager)
    }

    fn with_init(&mut self, f: impl FnOnce(&mut Self)) {
        if !Storage::is_initialized().unwrap() {
            self.logger.fatal(constants::NOT_INITIALIZED.as_ref());
        } else {
            f(self);
        }
    }

    fn handle_shield_up(&mut self) {
        if let Err(err) = Storage::create_dummy() {
            self.logger.error(&err);
            self.logger.fatal("Cannot create dummy directory".as_ref());
        }

        let dummy = match Storage::dummy() {
            Ok(data) => data,
            Err(err) => {
                self.logger.error(&err);
                self.logger
                    .fatal("Cannot get dummy directories' paths".as_ref());
            }
        };

        let root_dir = match Storage::root() {
            Ok(data) => data,
            Err(err) => {
                self.logger.error(&err);
                self.logger.fatal("Cannot get root path".as_ref());
            }
        };

        let honeypot_file = match Storage::upper_file() {
            Ok(path) => path,
            Err(err) => {
                self.logger.error(&err);
                self.logger.fatal("Cannot get honeypot file path".as_ref());
            }
        };

        let command = std::process::Command::new("mount")
            .arg("--bind")
            .arg(format!("{}", dummy.to_string_lossy()))
            .arg(format!("{}", root_dir.to_string_lossy()))
            .output();

        match command {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                self.logger.info(format!("{}\n", output.status).as_ref());
                self.logger.fatal("Cannot mount directory".as_ref());
            }
            Err(err) => {
                self.logger.error(&err);
                self.logger.fatal("Cannot mount directory".as_ref());
            }
        }

        self.logger
            .info("The shield is now up! Waiting for honeypot changes...\n".as_ref());
        let mut inotify = Inotify::init().unwrap();
        'outer: loop {
            inotify
                .watches()
                .add(
                    format!("{}", honeypot_file.to_string_lossy()),
                    WatchMask::OPEN,
                )
                .unwrap();

            let mut buffer = [0; 1024];
            let events = inotify.read_events_blocking(&mut buffer).unwrap();
            for _ in events {
                self.logger.info(
                    "The honeypot file has been touched! Triggering self-destruct\n".as_ref(),
                );
                self.handle_shield_down();
                std::thread::sleep(std::time::Duration::from_millis(1000));
                self.handle_clear();
                self.logger.info("All files have been deleted\n".as_ref());
                break 'outer;
            }
        }
    }

    fn handle_shield_down(&mut self) {
        let root_dir = match Storage::root() {
            Ok(data) => data,
            Err(err) => {
                self.logger.error(&err);
                self.logger.fatal("Cannot get root path".as_ref());
            }
        };

        let command = std::process::Command::new("umount")
            .arg(format!("{}", root_dir.to_string_lossy()))
            .spawn();

        match command {
            Ok(_) => {}
            Err(err) => {
                self.logger.error(&err);
                self.logger.fatal("Cannot mount overlayfs".as_ref());
            }
        }
        self.logger.info("The shield is now down!\n".as_ref());
    }
}
