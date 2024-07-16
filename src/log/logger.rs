use std::io::{Stdout, Write};

pub struct Logger<T>
where
    T: term::Terminal,
{
    terminal: T,
}

impl<T> Logger<T>
where
    T: term::Terminal,
{
    pub fn new(terminal: T) -> Self {
        Self { terminal }
    }

    pub fn info(&mut self, buf: &[u8]) {
        let _ = self.terminal.write(&buf);
    }

    pub fn warn(&mut self, buf: &[u8]) {
        let _ = self.terminal.fg(term::color::RED);
        let _ = self.terminal.write(&buf);
        let _ = self.terminal.reset();
    }

    pub fn fatal(&mut self, buf: &[u8]) -> ! {
        let _ = self.terminal.fg(term::color::RED);
        let _ = self.terminal.write(&buf);
        let _ = self.terminal.reset();
        std::process::exit(1);
    }

    pub fn flush(&mut self) {
        let _ = self.terminal.flush();
    }
}

impl Default for Logger<term::TerminfoTerminal<Stdout>> {
    fn default() -> Self {
        Self {
            terminal: term::TerminfoTerminal::new(std::io::stdout()).unwrap(),
        }
    }
}
