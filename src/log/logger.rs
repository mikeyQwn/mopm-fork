use std::{error::Error, io::Stdout};

pub struct Logger<T>
where
    T: term::Terminal,
{
    terminal: T,
    debug: bool,
}

#[cfg(debug_assertions)]
fn debug() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn debug() -> bool {
    false
}

impl<T> Logger<T>
where
    T: term::Terminal,
{
    pub fn new(terminal: T) -> Self {
        Self {
            terminal,
            debug: debug(),
        }
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn info(&mut self, buf: &[u8]) {
        let _ = self.terminal.write(buf);
    }

    pub fn warn(&mut self, buf: &[u8]) {
        let _ = self.terminal.fg(term::color::RED);
        let _ = self.terminal.write(buf);
        let _ = self.terminal.reset();
    }

    pub fn fatal(&mut self, buf: &[u8]) -> ! {
        let _ = self.terminal.fg(term::color::BRIGHT_RED);
        let _ = self.terminal.write(buf);
        let _ = self.terminal.reset();
        std::process::exit(1);
    }

    pub fn error(&mut self, error: &impl Error) {
        if !self.debug {
            return;
        }
        let _ = self.terminal.fg(term::color::RED);
        let _ = self.terminal.write(b"[DEBUG] error: ");
        let _ = self.terminal.write(error.to_string().as_ref());
        let _ = self.terminal.write(b"\n");
        let _ = self.terminal.reset();
    }

    pub fn flush(&mut self) {
        let _ = self.terminal.flush();
    }
}

impl Default for Logger<term::TerminfoTerminal<Stdout>> {
    fn default() -> Self {
        Self {
            terminal: term::TerminfoTerminal::new(std::io::stdout()).unwrap(),
            debug: debug(),
        }
    }
}
