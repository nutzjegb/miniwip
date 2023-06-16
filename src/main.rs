use crossterm::{
    execute,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    queue,
    Result,
    cursor,
    terminal::{
        self, size,
    },
    tty::IsTty,
    style::Print,
};
use clap::{Parser, crate_name, crate_version};
use futures::{future::FutureExt, select, StreamExt};
use std::path::PathBuf;
use std::io::{stdout, Write};
//use tokio::select;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    name: Option<String>,
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

//TODO move to print_start_stuff?
fn print_banner() -> Result<()> {
    let mut stdout = stdout();
    let is_tty = stdout.is_tty();
    let help = if is_tty { 
        "Press CTRL-A Z for help on special keys\n\n"
    } else { 
        "TTY not detected, fancy menus are disabled (hint use CTRL-A Q to quit)\n\n"
    };

    let banner = "Welcome to ".to_owned() + crate_name!() + " " + crate_version!() + "\n\nPort /dev/pts/0, 16:14:24\n" + help;

    //TODO print correct time
    //TODO print correct port

    execute!(stdout, Print(banner))?;
    Ok(())
}

fn print_startup_stuff() -> Result<()> {
    let mut stdout = stdout();
    let is_tty = stdout.is_tty();

    if is_tty {
        queue!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0))?;
    }
    print_banner()?;

    if is_tty {
        let (_cols, rows) = size()?;
        execute!(stdout,
            cursor::SavePosition,
            cursor::MoveTo(0, rows),
            Print("banner?"),
            cursor::RestorePosition)?;
    }
    Ok(())
}

fn exit() -> Result<()> {
    let mut stdout = stdout();
    let is_tty = stdout.is_tty();

    /* Print a newline as we don't know where the serial output ended */
    if is_tty {
        let (_cols, rows) = size()?;
        execute!(stdout,
            cursor::MoveTo(0, rows),
            Print(""))?;
    } else {
        println!("");
    }
    terminal::disable_raw_mode()?;
    Ok(())
}

fn print(buf: Vec<u8>) -> Result<()> {
    let mut stdout = stdout();

    //TODO if menu is displayed, buffer it
    //display it after the menu is closed

    // let utf8_str = String::from_utf8(buf)
    //     .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned());
    let str = String::from_utf8_lossy(&buf);
    execute!(stdout, Print(str))?;
    Ok(())
}

fn key_event_to_bytes(key_event: KeyEvent) -> Result<Option<Vec<u8>>> {
    //let mut vec: Vec<u8> = Vec::new();

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
        },
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

async fn event_handler(cli: Cli) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        //let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut input_event = reader.next().fuse();

        select! {
            //_ = delay => { println!(".\r"); },
            maybe_event = input_event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event {
                            /* Check for CTRL-A */
                            if let KeyCode::Char(c) = key_event.code {
                                if c == 'a' && key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
                                    //TODO implement app logic
                                    //for now quit
                                    break;
                                }
                            }

                            if let Some(data) = key_event_to_bytes(key_event)? {
                                print(data)?;
                            }
                        }
                    },
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }
    Ok(())
}

async fn main_app() -> Result<()> {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    println!("some dummy text 1");

    //TODO pass stdout instead
    //TODO handle errors

    print_startup_stuff()?;
    terminal::enable_raw_mode()?;

    let result = event_handler(cli).await;

    println!("end of");
    exit()?;

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    main_app().await?;
    Ok(())
}
