use crate::tui::Tui;
use crate::Cli;
use clap::{crate_name, crate_version};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::Result;
use std::io::Write;
use tokio_serial::SerialStream;

pub struct App {
    is_active: bool,
    tui: Tui,
    cli: Cli,
    status_delay: u64,
    add_carraige_return: AppOption<bool>,
    local_echo: AppOption<bool>,
    timestamp: AppOption<Timestamp>,
}

pub enum AppState {
    Quit,
    None,
}

enum AppOptions {
    LocalEcho,
    AddCarraigeReturn,
    Timestamp,
}

enum Timestamp {
    Simple,
    Extend,
    Off,
}

struct AppOption<T> {
    option: T,
    status_prefix: &'static str,
}
impl<T> AppOption<T>
where
    T: Copy,
{
    fn val(&self) -> T {
        self.option
    }
}
trait ToggleOption {
    fn toggle(&mut self);
    fn val_to_str(&self) -> &str;
    fn get_status_msg(&self) -> (&str, &str);
}
impl ToggleOption for AppOption<bool> {
    fn toggle(&mut self) {
        self.option = !self.option;
    }
    fn val_to_str(&self) -> &str {
        match self.option {
            true => "On",
            false => "Off",
        }
    }
    fn get_status_msg(&self) -> (&str, &str) {
        (self.status_prefix, self.val_to_str())
    }
}
impl ToggleOption for AppOption<Timestamp> {
    fn toggle(&mut self) {
        self.option = match self.option {
            Timestamp::Off => Timestamp::Simple,
            Timestamp::Simple => Timestamp::Extend,
            Timestamp::Extend => Timestamp::Off,
        }
    }
    fn val_to_str(&self) -> &str {
        match self.option {
            Timestamp::Off => "Off",
            Timestamp::Simple => "Simple",
            Timestamp::Extend => "Extended",
        }
    }
    fn get_status_msg(&self) -> (&str, &str) {
        (self.status_prefix, self.val_to_str())
    }
}

pub const TICKS_MS: u64 = 100;
const STATUS_DELAY_MS: u64 = 3000;
const STATUS_DELAY_TICKS: u64 = STATUS_DELAY_MS / TICKS_MS;

impl App {
    pub fn init(cli: Cli) -> Result<App> {
        let tui = Tui::init()?;

        let mut app = App {
            is_active: false,
            tui,
            cli,
            status_delay: 0,
            add_carraige_return: AppOption {
                option: false,
                status_prefix: "Add carraige return is ",
            },
            local_echo: AppOption {
                option: false,
                status_prefix: "Local echo is ",
            },
            timestamp: AppOption {
                option: Timestamp::Off,
                status_prefix: "Timestamp ",
            },
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
            "Press CTRL-A Z for help on special keys\r\n\n"
        } else {
            "TTY not detected, fancy menus are disabled (hint use CTRL-A Q to quit)\r\n\n"
        };

        let banner = "Welcome to ".to_owned()
            + crate_name!()
            + " "
            + crate_version!()
            + "\r\n\nPort "
            + &self.cli.device
            + ", 16:14:24\r\n"
            + help;

        //TODO print correct time

        self.tui.print(&banner)?;
        Ok(())
    }

    pub fn print_incoming(&mut self, buf: Vec<u8>) -> Result<()> {
        //wtf? waarom doet minicom \r ?
        if buf.contains(&b'\r') {
            println!("ah een r!\r");
        }

        let str = String::from_utf8_lossy(&buf);
        if self.add_carraige_return.val() && str.contains("\n") {
            println!("doe carraige return\r");
            self.tui.print(&str.replace("\n", "\r\n"))?;
        } else {
            self.tui.print(&str)?;
        }
        Ok(())
    }

    pub fn handle_serial_event(&mut self, data: &[u8]) -> Result<()> {
        // TODO replace with print incoming?
        self.print_incoming(data.to_vec())?;
        Ok(())
    }

    pub fn handle_key_event(
        &mut self,
        port: &mut SerialStream,
        key_event: KeyEvent,
    ) -> Result<AppState> {
        let mut result = AppState::None;

        if !self.is_active {
            /* Check for CTRL-A */
            if is_ctrl_a(key_event) {
                self.is_active = true;
                self.tui.set_status_msg("CTRL-A Z for help")?;
            } else {
                if let Some(data) = key_event_to_bytes(key_event)? {
                    port.write_all(&data)?;

                    if self.local_echo.val() {
                        self.print_incoming(data)?;
                    }
                }
            }
        } else {
            if is_ctrl_a(key_event) {
                /* Send the CTRL-A */
                if let Some(data) = key_event_to_bytes(key_event)? {
                    self.print_incoming(data)?;
                }
            } else if let KeyCode::Char(c) = key_event.code {
                match c {
                    'q' => result = AppState::Quit,
                    'x' => result = AppState::Quit,
                    'e' => self.toggle_option(AppOptions::LocalEcho)?,
                    'u' => self.toggle_option(AppOptions::AddCarraigeReturn)?,
                    'n' => self.toggle_option(AppOptions::Timestamp)?,
                    'c' => self.tui.clear_screen()?,
                    'z' => todo!(),
                    _ => (),
                }
            } else {
                // TODO
                /* Ignore other events? */
            }

            /* CTRL-A menu no longer active */
            self.is_active = false;

            /* Hide status when needed */
            if self.status_delay != STATUS_DELAY_TICKS {
                self.tui.hide_status()?;
            }
        }
        Ok(result)
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.cleanup()?;
        Ok(())
    }

    fn toggle_option(&mut self, option: AppOptions) -> Result<()> {
        let (prefix, val) = match option {
            AppOptions::AddCarraigeReturn => {
                // TODO I guess this could be one function..
                self.add_carraige_return.toggle();
                self.add_carraige_return.get_status_msg()
            }
            AppOptions::LocalEcho => {
                self.local_echo.toggle();
                self.add_carraige_return.get_status_msg()
            }
            AppOptions::Timestamp => {
                self.timestamp.toggle();
                self.add_carraige_return.get_status_msg()
            }
        };
        self.tui.set_status(prefix, val)?;
        self.status_delay = STATUS_DELAY_TICKS;

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

    /* Note, rust does not has some C-escape codes like \b or \e */
    let key_str: Option<Vec<u8>> = match key_event.code {
        KeyCode::Backspace => Some(Vec::from([b'\x08'])),
        KeyCode::Enter => Some(Vec::from([b'\r', b'\n'])),
        KeyCode::Left => todo!(),
        KeyCode::Right => todo!(),
        KeyCode::Up => todo!(),
        KeyCode::Down => todo!(),
        KeyCode::Home => todo!(),
        KeyCode::End => todo!(),
        KeyCode::PageUp => todo!(),
        KeyCode::PageDown => todo!(),
        KeyCode::Tab => Some(Vec::from([b'\t'])),
        KeyCode::BackTab => todo!(),
        KeyCode::Delete => todo!(),
        KeyCode::Insert => todo!(),
        KeyCode::F(_) => todo!(),
        KeyCode::Char(ch) => {
            //TODO check
            if is_ctrl_key(key_event) && ch >= 'a' && ch <= 'f' {
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
        KeyCode::KeypadBegin => todo!(),
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
