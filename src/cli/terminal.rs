pub struct Terminal;

impl Terminal {}

impl Terminal {
    pub fn read_password() -> String {
        rpassword::read_password().unwrap()
    }
}
