use crate::tui::Tui;
use crate::Cli;
use clap::{crate_name, crate_version};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::Result;

pub struct App {
    is_active: bool,
    tui: Tui,
    cli: Cli,
    status_delay: u64,
    add_carraige_return: bool,
}

pub enum AppState {
    Quit,
    None,
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
            add_carraige_return: false,
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
        if buf[0] == b'\n' && self.add_carraige_return {
            self.tui.print("\r")?;
        }

        let str = String::from_utf8_lossy(&buf);
        self.tui.print(&str)?;
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<AppState> {
        let mut result = AppState::None;

        if !self.is_active {
            /* Check for CTRL-A */
            if is_ctrl_a(key_event) {
                self.is_active = true;
                self.tui.set_status_msg("banner time!")?;
            } else {
                if let Some(data) = key_event_to_bytes(key_event)? {
                    self.print_incoming(data)?;
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
                    'u' => self.toggle_carraige_return()?,
                    'c' => self.tui.clear_screen()?,
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

    //TODO
    //mm something like this?
    // fn toggle_item(&mut self, item: &mut bool, msg: &str) -> Result<()> {
    //     *item = !*item;
    //     self.tui.set_status(msg, self.add_carraige_return)?;
    //     self.status_delay = STATUS_DELAY_TICKS;
    //     Ok(())
    // }

    fn toggle_carraige_return(&mut self) -> Result<()> {
        self.add_carraige_return = !self.add_carraige_return;
        let prefix = "Carraige return";
        self.tui.set_status(prefix, self.add_carraige_return)?;
        self.status_delay = STATUS_DELAY_TICKS;
        Ok(())
    }
}

fn key_event_to_bytes(key_event: KeyEvent) -> Result<Option<Vec<u8>>> {
    let esc: u8 = b'\x1b';

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
            //TODO
            //if key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
            //}
            Some(Vec::from([ch as u8]))
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

fn is_ctrl_a(key_event: KeyEvent) -> bool {
    if let KeyCode::Char(c) = key_event.code {
        if c == 'a' && key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
            return true;
        }
    }
    false
}
