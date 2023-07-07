use crate::app::Timestamp;
use anyhow::Result;
use crossterm::{cursor, execute, queue, style::Print, terminal, tty::IsTty};
use std::io::{stdout, Write};
use time::{macros::format_description, OffsetDateTime};

pub struct Tui {
    is_tty: bool,
    stdout: std::io::Stdout,
    status_msg: String,

    on_newline: bool,
    prefix_timestamp: Timestamp,

    cur_row: u16,
}

const FORMAT_SIMPLE: &[time::format_description::FormatItem<'static>] =
    format_description!(version = 2, r"\[[hour]:[minute]:[second]\] ");
const FORMAT_EXTENDED: &[time::format_description::FormatItem<'static>] = format_description!(
    version = 2,
    r"\[[hour]:[minute]:[second].[subsecond digits:3]\] "
);

struct PrintTime(
    pub OffsetDateTime,
    pub &'static [time::format_description::FormatItem<'static>],
);
impl crossterm::Command for PrintTime {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let s = self.0.format(self.1).unwrap();
        f.write_str(&s)
    }
}

impl Tui {
    pub fn init() -> Result<Tui> {
        let stdout = stdout();
        terminal::enable_raw_mode()?;

        Ok(Tui {
            is_tty: stdout.is_tty(),
            stdout,
            status_msg: "".to_string(),
            on_newline: false,
            prefix_timestamp: Timestamp::Off,
            cur_row: 0,
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        /* Print a newline as we don't know where the serial output ended */
        if self.is_tty {
            let (_cols, rows) = terminal::size()?;
            execute!(self.stdout, cursor::MoveTo(0, rows - 1), Print("\r\n"))?;
        } else {
            print!("\r\n");
        }

        Ok(())
    }

    pub fn is_tty(&mut self) -> bool {
        self.is_tty
    }

    fn handle_last_line(&mut self) -> Result<()> {
        let (_cols, rows) = terminal::size()?;

        // TODO handle resize
        // for now just accept things look weird when resizing

        // TODO this is bound to break
        // Fix cursor::position() somehow?
        // (it blocks and fails sometimes..)
        if self.cur_row >= rows - 1 {
            queue!(
                self.stdout,
                terminal::Clear(terminal::ClearType::UntilNewLine),
                cursor::SavePosition,
                terminal::ScrollUp(1),
                cursor::MoveTo(0, rows - 1),
                Print(&self.status_msg),
                cursor::RestorePosition,
                cursor::MoveUp(1),
            )?;
            self.cur_row = rows - 1;
        }
        Ok(())
    }

    pub fn print(&mut self, str: &str) -> Result<()> {
        // TODO OffsetDateTime::now_local() fails as it is not thread safe
        let time = OffsetDateTime::now_utc();

        let split = str.split_inclusive('\n');
        for line in split {
            if self.on_newline && self.prefix_timestamp != Timestamp::Off {
                let format = match self.prefix_timestamp {
                    Timestamp::Simple => FORMAT_SIMPLE,
                    Timestamp::Extend => FORMAT_EXTENDED,
                    Timestamp::Off => unreachable!(),
                };
                queue!(self.stdout, PrintTime(time, format), Print(line))?;
            } else {
                queue!(self.stdout, Print(line))?;
            }

            if line.ends_with('\n') {
                self.on_newline = true;
                self.cur_row += 1;
            } else {
                self.on_newline = false;
            }
            // Handle the last line so we don't overwrite the status line
            self.handle_last_line()?;
        }
        self.stdout.flush()?;

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
        self.cur_row = 0;
        Ok(())
    }

    pub fn set_prefix_timestamp(&mut self, timestamp: Timestamp) {
        self.prefix_timestamp = timestamp;
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        /* Ignore error here */
        let _ = terminal::disable_raw_mode();
    }
}
