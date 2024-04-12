use crate::tui::Tui;
use crate::Cli;
use anyhow::Result;
use clap::{crate_name, crate_version};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::Write;
use tokio_serial::SerialStream;

// TODO add support for logging to a file?
// TODO add support to paste a file?
// TODO Allow setting the serialport options? (or cmdline only)

#[derive(Clone, Copy, PartialEq)]
enum Commands {
    Quit,
    Exit,
    ToggleLocalEcho,
    ToggleLineFeed,
    ToggleCarriageReturn,
    ToggleTimestamp,
    ClearScreen,
    ShowHelp,
}
// impl Commands {
//     fn key(&self) -> char {
//         use Commands::*;
//         match *self {
//             Quit => 'q',
//             Exit => 'x',
//             ToggleLocalEcho => 'e',
//             ToggleLineFeed => 'a',
//             ToggleCarriageReturn => 'u',
//             ToggleTimestamp => 'n',
//             ClearScreen => 'c',
//             ShowHelp => 'z',
//         }
//     }
//     fn desc(&self) -> &'static str {
//         return "test";
//     }
// }

trait OptionAsString {
    fn val_to_str(&self) -> &'static str;
}
impl OptionAsString for bool {
    fn val_to_str(&self) -> &'static str {
        if *self {
            "True"
        } else {
            "False"
        }
    }
}
impl OptionAsString for Timestamp {
    fn val_to_str(&self) -> &'static str {
        match *self {
            Timestamp::Simple => "Simple",
            Timestamp::Extend => "Extended",
            Timestamp::Off => "Off",
        }
    }
}

fn get_command(cmd: char) -> Option<Commands> {
    use Commands::*;
    match cmd {
        'q' => Some(Quit),
        'x' => Some(Exit),
        'e' => Some(ToggleLocalEcho),
        'a' => Some(ToggleLineFeed),
        'u' => Some(ToggleCarriageReturn),
        'n' => Some(ToggleTimestamp),
        'c' => Some(ClearScreen),
        'z' => Some(ShowHelp),
        _ => None,
    }
}
// #[cfg(test)]
// mod tests {
//     use crate::app::get_command;

//     #[test]
//     fn matches() {
//         for c in 'a'..'z' {
//             if let Some(cmd) = get_command(c) {
//                 assert!(cmd.key() == c);
//             }
//         }
//     }
// }

pub struct App {
    state: AppStates,
    tui: Tui,
    cli: Cli,
    status_delay: u64,
    add_carriage_return: bool,
    add_line_feed: bool,
    local_echo: bool,
    timestamp: Timestamp,
}

pub enum AppStates {
    Receiving,
    MenuActive,
    CatchKey,
}

pub enum AppResults {
    Quit,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Timestamp {
    Simple,
    Extend,
    Off,
}
impl Timestamp {
    fn next(&self) -> Self {
        match *self {
            Timestamp::Simple => Timestamp::Extend,
            Timestamp::Extend => Timestamp::Simple,
            Timestamp::Off => Timestamp::Simple,
        }
    }
}

// struct AppOption<T> {
//     option: T,
//     status_prefix: &'static str,
// }
// impl<T> AppOption<T>
// where
//     T: Copy,
// {
//     fn val(&self) -> T {
//         self.option
//     }
// }
// trait ToggleOption {
//     fn toggle(&mut self);
//     fn val_to_str(&self) -> &str;
//     fn toggle_and_get_status_msg(&mut self) -> (&str, &str);
// }
// impl ToggleOption for AppOption<bool> {
//     fn toggle(&mut self) {
//         self.option = !self.option;
//     }
//     fn val_to_str(&self) -> &str {
//         match self.option {
//             true => "On",
//             false => "Off",
//         }
//     }
//     fn toggle_and_get_status_msg(&mut self) -> (&str, &str) {
//         self.toggle();
//         (self.status_prefix, self.val_to_str())
//     }
// }
// impl ToggleOption for AppOption<Timestamp> {
//     fn toggle(&mut self) {
//         self.option = match self.option {
//             Timestamp::Off => Timestamp::Simple,
//             Timestamp::Simple => Timestamp::Extend,
//             Timestamp::Extend => Timestamp::Off,
//         }
//     }
//     fn val_to_str(&self) -> &str {
//         match self.option {
//             Timestamp::Off => "Off",
//             Timestamp::Simple => "Simple",
//             Timestamp::Extend => "Extended",
//         }
//     }
//     fn toggle_and_get_status_msg(&mut self) -> (&str, &str) {
//         self.toggle();
//         (self.status_prefix, self.val_to_str())
//     }
// }

pub const TICKS_MS: u64 = 100;
const STATUS_DELAY_MS: u64 = 3000;
const STATUS_DELAY_TICKS: u64 = STATUS_DELAY_MS / TICKS_MS;

impl App {
    pub fn init(cli: Cli) -> Result<App> {
        let tui = Tui::init()?;

        let mut app = App {
            state: AppStates::Receiving,
            tui,
            cli,
            status_delay: 0,
            add_carriage_return: false,
            add_line_feed: false,
            local_echo: false,
            timestamp: Timestamp::Off,
        };
        app.print_startup_stuff()?;

        Ok(app)
    }

    pub fn tick(&mut self) -> Result<()> {
        if self.status_delay != 0 {
            self.status_delay -= 1;
            if self.status_delay == 0 {
                self.tui.hide_status()?;
            }
        }
        Ok(())
    }

    fn print_startup_stuff(&mut self) -> Result<()> {
        if self.tui.is_tty() {
            self.tui.clear_screen()?;
        }

        let help = if self.tui.is_tty() {
            "Press CTRL-A Z for help on special keys\r\n\r\n"
        } else {
            "TTY not detected, fancy menus are disabled (hint use CTRL-A Q to quit)\r\n\r\n"
        };

        let banner = "Welcome to ".to_owned()
            + crate_name!()
            + " "
            + crate_version!()
            + "\r\n\r\nPort "
            + &self.cli.device
            + "\r\n"
            + help;

        self.tui.print_to_screen(&banner)?;
        Ok(())
    }

    fn show_help(&mut self) -> Result<()> {
        if !self.tui.is_tty() {
            return Ok(());
        }

        let msg = "help help help\n\r";

        self.tui.enter_alt()?;
        self.state = AppStates::MenuActive;
        self.tui.print_to_screen(msg)?;

        Ok(())
    }

    fn print_incoming(&mut self, buf: &Vec<u8>) -> Result<()> {
        // TODO refactor vec to u8

        let str = String::from_utf8_lossy(&buf);

        //crappy hex
        // let dg = str.as_bytes();
        // let bla = format!("{dg:x?}\n\r");
        // self.tui.print_or_queue(&bla)?;

        // TODO instead of replace, use split?
        if self.add_carriage_return && str.contains('\n') {
            self.tui.print_or_queue(&str.replace('\n', "\r\n"))?;
        } else if self.add_line_feed && str.contains('\r') {
            self.tui.print_or_queue(&str.replace('\r', "\r\n"))?;
        } else {
            self.tui.print_or_queue(&str)?;
        }
        Ok(())
    }

    pub fn handle_serial_event(&mut self, data: &[u8]) -> Result<()> {
        self.print_incoming(&data.to_vec())?;
        Ok(())
    }

    fn send_serial_data(&mut self, port: &mut SerialStream, data: &Vec<u8>) -> Result<()> {
        port.write_all(&data)?;
        if self.local_echo {
            self.print_incoming(data)?;
        }
        Ok(())
    }

    fn handle_command(&mut self, cmd: Commands) -> Result<AppResults> {
        let mut result = AppResults::None;

        match cmd {
            Commands::Quit | Commands::Exit => result = AppResults::Quit,
            Commands::ToggleLocalEcho => {
                self.local_echo = !self.local_echo;
                self.tui.set_status("bla", self.local_echo.val_to_str())?;
            },
            Commands::ToggleLineFeed => {
                self.add_line_feed = !self.add_line_feed;
                self.tui.set_status("bla", self.add_line_feed.val_to_str())?;
            },
            Commands::ToggleCarriageReturn => {
                self.add_carriage_return = !self.add_carriage_return;
                self.tui.set_status("bla", self.add_carriage_return.val_to_str())?;
            },
            Commands::ToggleTimestamp => {
                self.timestamp = self.timestamp.next();
                self.tui.set_prefix_timestamp(self.timestamp);
                self.tui.set_status("sdfg", self.timestamp.val_to_str())?;
            },
            Commands::ClearScreen => self.tui.clear_screen()?,
            Commands::ShowHelp => self.show_help()?,
        }

        Ok(result)
    }

    pub fn handle_key_event(
        &mut self,
        port: &mut SerialStream,
        key_event: KeyEvent,
    ) -> Result<AppResults> {
        let mut result = AppResults::None;

        match self.state {
            AppStates::Receiving => {
                assert!(!self.tui.on_alternate_screen());

                /* Check for CTRL-A */
                if is_ctrl_a(key_event) {
                    self.state = AppStates::CatchKey;
                    self.tui.set_status_msg("CTRL-A Z for help")?;
                } else if let Some(data) = key_event_to_bytes(key_event)? {
                    self.send_serial_data(port, &data)?;
                    // TODO: add separate option?
                    // (currently like minicom, one option for both receiving and sending)
                    if self.add_line_feed && data[0] == b'\r' {
                        self.send_serial_data(port, &vec![b'\n'])?;
                    }
                }
            },
            AppStates::CatchKey => {
                /* Leave the state */
                self.state = AppStates::Receiving;

                if is_ctrl_a(key_event) {
                    /* Got CTRL-A for the second time, send it */
                    if let Some(data) = key_event_to_bytes(key_event)? {
                        self.send_serial_data(port, &data)?;
                    }
                } else if let KeyCode::Char(c) = key_event.code {
                    if let Some(cmd) = get_command(c) {
                        result = self.handle_command(cmd)?;
                    }
                } else {
                    /* Ignore other keys like 'enter' */
                }

                /* Hide status when needed */
                if self.status_delay != STATUS_DELAY_TICKS {
                    self.tui.hide_status()?;
                }
            },
            AppStates::MenuActive => {
                /* For now, leave the menu on any key */
                self.tui.leave_alt()?;
                self.state = AppStates::Receiving;
            },
        }
        Ok(result)
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.cleanup()?;
        Ok(())
    }
}

fn key_event_to_bytes(key_event: KeyEvent) -> Result<Option<Vec<u8>>> {
    let esc: u8 = b'\x1b';

    // TODO instead of vec?
    // let mut buf: [u8; 4] = [0; 4];
    // let test: u32 = 0x1b;
    // buf = u32::to_ne_bytes(test << 16 | 1);
    // return size?
    // or wrapped in some struct

    // TODO verify against u-boot or something similar
    let key_str: Option<Vec<u8>> = match key_event.code {
        KeyCode::Backspace => Some(Vec::from([b'\x08'])),
        KeyCode::Enter => Some(Vec::from([b'\r'])),
        KeyCode::Left => Some(Vec::from([esc, b'\x5b', b'\x44'])),
        KeyCode::Right => Some(Vec::from([esc, b'\x5b', b'\x43'])),
        KeyCode::Up => Some(Vec::from([esc, b'\x5b', b'\x41'])),
        KeyCode::Down => Some(Vec::from([esc, b'\x5b', b'\x42'])),
        KeyCode::Home => Some(Vec::from([esc, b'\x5b', b'\x48'])),
        KeyCode::End => Some(Vec::from([esc, b'\x5b', b'\x46'])),
        KeyCode::PageUp => Some(Vec::from([esc, b'\x5b', b'\x35', b'\x7e'])),
        KeyCode::PageDown => Some(Vec::from([esc, b'\x5b', b'\x36', b'\x7e'])),
        KeyCode::Tab => Some(Vec::from([b'\t'])),
        KeyCode::BackTab => Some(Vec::from([esc, b'\x5b', b'\x5a'])),
        KeyCode::Delete => Some(Vec::from([esc, b'\x5b', b'\x33', b'\x7e'])),
        KeyCode::Insert => Some(Vec::from([esc, b'\x5b', b'\x32', b'\x7e'])),
        KeyCode::F(num) => {
            if num <= 4 {
                Some(Vec::from([esc, b'\x5b', b'\x4f', b'\x49' + num]))
            } else {
                None
            }
        }
        KeyCode::Char(ch) => {
            if is_ctrl_key(key_event) && ch.is_ascii_lowercase() {
                Some(Vec::from([ch as u8 - b'a']))
            } else {
                Some(Vec::from([ch as u8]))
            }
        }
        KeyCode::Null => Some(Vec::from([b'\0'])),
        KeyCode::Esc => Some(Vec::from([esc])),
        KeyCode::CapsLock => None,
        KeyCode::ScrollLock => None,
        KeyCode::NumLock => None,
        KeyCode::PrintScreen => None,
        KeyCode::Pause => None,
        KeyCode::Menu => None,
        KeyCode::KeypadBegin => None,
        KeyCode::Media(_) => None,
        KeyCode::Modifier(_) => None,
    };
    Ok(key_str)
}

fn is_ctrl_key(key_event: KeyEvent) -> bool {
    if key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
        return true;
    }
    false
}

fn is_ctrl_a(key_event: KeyEvent) -> bool {
    if let KeyCode::Char(c) = key_event.code {
        if c == 'a' && key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
            return true;
        }
    }
    false
}
