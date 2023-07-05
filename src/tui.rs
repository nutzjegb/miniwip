use crossterm::{cursor, execute, queue, style::Print, terminal, tty::IsTty, Result};
use std::io::{stdout, Write};

pub struct Tui {
    is_tty: bool,
    stdout: std::io::Stdout,
    status_msg: String,
}

impl Tui {
    pub fn init() -> Result<Tui> {
        let stdout = stdout();
        terminal::enable_raw_mode()?;

        Ok(Tui {
            is_tty: stdout.is_tty(),
            stdout,
            status_msg: "".to_string(),
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        /* Print a newline as we don't know where the serial output ended */
        if self.is_tty {
            let (_cols, rows) = terminal::size()?;
            execute!(self.stdout, cursor::MoveTo(0, rows - 1), Print("\r\n"))?;
        } else {
            println!("");
        }
        terminal::disable_raw_mode()?;

        Ok(())
    }

    pub fn is_tty(&mut self) -> bool {
        self.is_tty
    }

    fn handle_last_line(&mut self) -> Result<()> {
        let (_cols, rows) = terminal::size()?;
        let (col, row) = cursor::position()?;

        if row == rows - 1 {
            queue!(
                self.stdout,
                terminal::Clear(terminal::ClearType::UntilNewLine),
                terminal::ScrollUp(1),
                cursor::MoveTo(0, rows - 1),
                Print(&self.status_msg),
                cursor::MoveTo(col, row - 1),
            )?;
        }
        Ok(())
    }

    pub fn queue(&mut self, char: u8) -> Result<()> {
        queue!(self.stdout, Print(char))?;
        if self.is_tty && char == b'\n' {
            self.handle_last_line()?;
        }

        Ok(())
    }

    pub fn print(&mut self, str: &str) -> Result<()> {
        if self.is_tty && str.contains('\n') {
            queue!(self.stdout, Print(str))?;
            self.handle_last_line()?;
            self.stdout.flush()?;
        } else {
            execute!(self.stdout, Print(str))?;
        }
        Ok(())
    }

    pub fn set_status_msg(&mut self, str: &str) -> Result<()> {
        if self.is_tty {
            let (_cols, rows) = terminal::size()?;
            execute!(
                self.stdout,
                cursor::SavePosition,
                cursor::MoveTo(0, rows - 1),
                terminal::Clear(terminal::ClearType::CurrentLine),
                Print(str),
                cursor::RestorePosition
            )?;

            self.status_msg = str.to_string();
        }
        Ok(())
    }

    pub fn set_status(&mut self, prefix: &str, val: &str) -> Result<()> {
        if self.is_tty {
            // TODO fix set_status_msg api
            let msg = prefix.to_owned() + val;
            self.set_status_msg(&msg)?;
        }
        Ok(())
    }

    pub fn hide_status(&mut self) -> Result<()> {
        if self.is_tty {
            let (_cols, rows) = terminal::size()?;
            execute!(
                self.stdout,
                cursor::SavePosition,
                cursor::MoveTo(0, rows - 1),
                terminal::Clear(terminal::ClearType::CurrentLine),
                cursor::RestorePosition
            )?;

            self.status_msg = "".to_string();
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
