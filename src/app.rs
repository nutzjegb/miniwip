use crate::tui::Tui;
use clap::{crate_name, crate_version};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::Result;

pub struct App {
    is_active: bool,
    tui: Tui,
}

pub enum AppState {
    Quit,
    None,
}

impl App {
    pub fn init() -> Result<App> {
        let tui = Tui::init()?;

        let mut app = App {
            is_active: false,
            tui,
        };
        app.print_startup_stuff()?;

        Ok(app)
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
            + "\r\n\nPort /dev/pts/0, 16:14:24\r\n"
            + help;

        //TODO print correct time
        //TODO print correct port

        self.tui.print(&banner)?;
        Ok(())
    }

    pub fn print_incoming(&mut self, buf: Vec<u8>) -> Result<()> {
        let str = String::from_utf8_lossy(&buf);
        self.tui.print(&str)?;
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<AppState> {
        if !self.is_active {
            /* Check for CTRL-A */
            if is_ctrl_a(key_event) {
                self.is_active = true;
                self.tui.set_status("banner time!")?;
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
                    'q' => return Ok(AppState::Quit),
                    'x' => return Ok(AppState::Quit),
                    'c' => self.tui.clear_screen()?,
                    _ => (),
                }
            } else {
                // TODO
                /* Ignore other events? */
            }

            self.is_active = false;
            self.tui.hide_status()?;
        }
        Ok(AppState::None)
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.cleanup()?;
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
