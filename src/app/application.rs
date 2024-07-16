use crate::{
    cli::{
        config::{Command, Config},
        terminal::Terminal,
    },
    core::{
        encoder::{Encoder, EncoderError},
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
        let command = std::mem::replace(&mut self.config.command, Command::Init);

        match command {
            Command::Init => self.handle_init(),
            Command::Clear => self.handle_clear(),
            Command::Store(key, value) => self.handle_store(key.as_ref(), value.as_ref()),
            Command::Get(key) => self.handle_get(key.as_ref()),
        }
    }

    fn handle_init(&mut self) {
        let password = self.prompt_password();
        let mut pm = PasswordManager::init(password.trim());

        match Storage::init(&mut pm) {
            Ok(_) => self.logger.info(constants::INIT_SUCCESSFULL.as_ref()),
            Err(StorageError::RootAlreadyExistsErorr) => {
                self.logger.warn(constants::ALREADY_INITIALIZED.as_ref())
            }
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
        let mut pm_reader = Storage::get_data_reader().unwrap();
        Encoder::decode(password.trim().as_ref(), &mut pm_reader).unwrap()
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
}
