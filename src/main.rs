use crossterm::{
    execute,
    event::{Event, EventStream, KeyCode},
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

//fn handle_key_event(key_event: KeyEvent, opt: &Opt) -> Result<Option<Bytes>> {
//    Ok(None)
//}

fn print(c: char) -> Result<()> {
    let mut stdout = stdout();

    //TODO is menu is display, buffer it
    //display it after the menu is closed

    execute!(stdout, Print(c))?;
    Ok(())
}

async fn event_handler(cli: Cli) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        //let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();

        select! {
            //_ = delay => { println!(".\r"); },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event {
                            match key_event.code {
                                KeyCode::Esc => break,
                                    KeyCode::Enter => {
                                        //TODO print buffer
                                        print('\n')?;
                                        print('\r')?;
                                    },
                                    KeyCode::Char(ch) => {
                                        if ch == 'q' {
                                            break;
                                        } else {
                                            print(ch)?;
                                        }
                                    },
                                    _ => {
                                        println!("uncaught keycode?: {:?}", key_event);
                                        break;
                                    },
                            }
                        } else {
                            println!("Unknown event? {:?}", event);
                        }
                        //if event == Event::Key(KeyCode::Enter.into()) {
                        //    print('\n')?;
                        //    print('\r')?;
                        //}
                        //else if event == Event::Key(KeyCode::Esc.into()) {
                        //    break;
                        //}
                        //else if event == Event::Key(KeyCode::Char('q').into()) {
                        //    break;
                        //}
                        //else if let Event::Key(key_event) = event {
                        //    
                        //    //println!("Event::{:?}\r", event);
                        //    //print(key_event.code)?;
                        //}
                    }
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
