use crate::app::Timestamp;
use anyhow::Result;
use crossterm::{cursor, execute, queue, style::Print, terminal, tty::IsTty};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, Paragraph}, Frame, Terminal};
use std::{collections::VecDeque, io::{stdout, Stdout, Write}};
use time::{macros::format_description, OffsetDateTime};

struct ToPrint {
    time: OffsetDateTime,
    str: String,
}

pub struct Tui {
    is_tty: bool,
    stdout: std::io::Stdout,
    // status_msg: String,
    on_alternate_screen: bool,

    on_newline: bool,
    prefix_timestamp: Timestamp,

    queue: VecDeque<ToPrint>,

    // cur_row: u16,
    terminal: Terminal<CrosstermBackend<Stdout>>,
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
        let out = stdout();
        // TODO: Always enable raw? or only on is_tty? (cant remember)
        terminal::enable_raw_mode()?;

        let term = Terminal::new(CrosstermBackend::new(stdout()))?;

        Ok(Tui {
            is_tty: out.is_tty(),
            stdout: out,
            // status_msg: "".to_string(),
            on_alternate_screen: false,
            on_newline: false,
            prefix_timestamp: Timestamp::Off,
            // cur_row: 0,
            queue: VecDeque::new(),
            terminal: term,
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.leave_alt()?;

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

    // fn handle_last_line(&mut self) -> Result<()> {
    //     let (_cols, rows) = terminal::size()?;

    //     // TODO handle resize
    //     // for now just accept things look weird when resizing

    //     // TODO this is bound to break
    //     // Fix cursor::position() somehow?
    //     // (it blocks and fails sometimes..)
    //     if self.cur_row >= rows - 1 {
    //         queue!(
    //             self.stdout,
    //             terminal::Clear(terminal::ClearType::UntilNewLine),
    //             cursor::SavePosition,
    //             terminal::ScrollUp(1),
    //             cursor::MoveTo(0, rows - 1),
    //             Print(&self.status_msg),
    //             cursor::RestorePosition,
    //             cursor::MoveUp(1),
    //         )?;
    //         self.cur_row = rows - 1;
    //     }
    //     Ok(())
    // }

    fn flush_print_queue(&mut self) -> Result<()> {
        while !self.queue.is_empty() {
            let item = self.queue.pop_front().unwrap();
            self.print_line(item.time, &item.str)?;
        }
        Ok(())
    }

    fn print_line(&mut self, time: OffsetDateTime, str: &str) -> Result<()> {
        assert!(!self.on_alternate_screen);

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
                // self.cur_row += 1;
            } else {
                self.on_newline = false;
            }
            // Handle the last line so we don't overwrite the status line
            // self.handle_last_line()?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    pub fn print_to_screen(&mut self, str: &str) -> Result<()> {
        assert!(!self.on_alternate_screen);

        execute!(self.stdout, Print(str))?;
        Ok(())
    }

    pub fn print_or_queue(&mut self, str: &str) -> Result<()> {
        // TODO OffsetDateTime::now_local() fails as it is not thread safe
        let time = OffsetDateTime::now_utc();

        if self.on_alternate_screen {
            self.queue.push_back(ToPrint {time, str: str.to_string()});
        } else {
            self.print_line(time, str)?;
        }

        Ok(())
    }

    pub fn set_status_msg(&mut self, _str: &str) -> Result<()> {
        // if self.is_tty {
        //     let (_cols, rows) = terminal::size()?;
        //     execute!(
        //         self.stdout,
        //         cursor::SavePosition,
        //         cursor::MoveTo(0, rows - 1),
        //         terminal::Clear(terminal::ClearType::CurrentLine),
        //         Print(str),
        //         cursor::RestorePosition
        //     )?;

        //     self.status_msg = str.to_string();
        // }
        Ok(())
    }

    pub fn set_status(&mut self, _prefix: &str, _val: &str) -> Result<()> {
        // if self.is_tty {
        //     // TODO fix set_status_msg api
        //     let msg = prefix.to_owned() + val;
        //     self.set_status_msg(&msg)?;
        // }
        Ok(())
    }

    pub fn hide_status(&mut self) -> Result<()> {
        // if self.is_tty {
        //     let (_cols, rows) = terminal::size()?;
        //     execute!(
        //         self.stdout,
        //         cursor::SavePosition,
        //         cursor::MoveTo(0, rows - 1),
        //         terminal::Clear(terminal::ClearType::CurrentLine),
        //         cursor::RestorePosition
        //     )?;

        //     self.status_msg = "".to_string();
        // }
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        assert!(!self.on_alternate_screen);

        execute!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        // self.cur_row = 0;
        Ok(())
    }

    pub fn set_prefix_timestamp(&mut self, timestamp: Timestamp) {
        self.prefix_timestamp = timestamp;
    }

    pub fn enter_alt(&mut self) -> Result<()> {
        if !self.on_alternate_screen {
            execute!(
                self.stdout,
                cursor::SavePosition,
                terminal::EnterAlternateScreen,
                cursor::Hide,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
            self.on_alternate_screen = true;
            self.terminal.clear()?;
        }
        Ok(())
    }

    pub fn leave_alt(&mut self) -> Result<()> {
        if self.on_alternate_screen {
            execute!(
                self.stdout,
                terminal::LeaveAlternateScreen,
                cursor::RestorePosition,
                cursor::Show,
            )?;
            self.on_alternate_screen = false;
            self.flush_print_queue()?;
        }
        Ok(())
    }

    pub fn on_alternate_screen(&self) -> bool {
        self.on_alternate_screen
    }

    pub fn draw_ui(&mut self) -> Result<()> {
        assert!(self.on_alternate_screen);
        self.terminal.draw(ui)?;
        Ok(())
    }

    pub fn resize(&mut self) -> Result<()> {
        assert!(self.on_alternate_screen);
        self.terminal.autoresize()?;
        Ok(())
    }
}

fn ui(frame: &mut Frame) {
    // frame.render_widget(
    //     Paragraph::new("Hello World!")
    //         .block(Block::default().title("Greeting").borders(Borders::ALL)),
    //     frame.size(),
    // );
    let inner_layout = Layout::new(
        Direction::Horizontal, [
            Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)],
    ).split(frame.size());
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Left"),
        inner_layout[0],
    );
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("mid"),
        inner_layout[1],
    );
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Right"),
        inner_layout[2],
    );
}

impl Drop for Tui {
    fn drop(&mut self) {
        /* Ignore errors here */
        // TODO: move to cleanup func or something?
        // (logic is a bit entangled)
        // TODO: if self.is_tty ?
        let _ = self.leave_alt();
        let _ = terminal::disable_raw_mode();
    }
}
