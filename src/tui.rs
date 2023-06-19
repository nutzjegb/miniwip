use crossterm::{
    cursor, execute, queue,
    style::Print,
    terminal::{self, size},
    tty::IsTty,
    Result,
};
use std::io::{stdout, Write};

pub struct Tui {
    is_tty: bool,
    stdout: std::io::Stdout,
}

impl Tui {
    pub fn init() -> Result<Tui> {
        let stdout = stdout();
        terminal::enable_raw_mode()?;

        Ok(Tui {
            is_tty: stdout.is_tty(),
            stdout,
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        /* Print a newline as we don't know where the serial output ended */
        if self.is_tty {
            let (_cols, rows) = size()?;
            execute!(self.stdout, cursor::MoveTo(0, rows), Print("\r\n"))?;
        } else {
            println!("");
        }
        terminal::disable_raw_mode()?;

        Ok(())
    }

    pub fn is_tty(&mut self) -> bool {
        self.is_tty
    }

    pub fn print(&mut self, str: &str) -> Result<()> {
        execute!(self.stdout, Print(str))
    }

    pub fn set_status(&mut self, str: &str) -> Result<()> {
        if self.is_tty {
            let (_cols, rows) = size()?;
            execute!(
                self.stdout,
                cursor::SavePosition,
                cursor::MoveTo(0, rows),
                terminal::Clear(terminal::ClearType::CurrentLine),
                Print(str),
                cursor::RestorePosition
            )?;
        }
        Ok(())
    }

    pub fn hide_status(&mut self) -> Result<()> {
        if self.is_tty {
            let (_cols, rows) = size()?;
            execute!(
                self.stdout,
                cursor::SavePosition,
                cursor::MoveTo(0, rows),
                terminal::Clear(terminal::ClearType::CurrentLine),
                cursor::RestorePosition
            )?;
        }
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }
}
